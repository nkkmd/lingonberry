#!/usr/bin/env python3
"""Real-host executor for the Lingonberry v1.0 soak schedule.

This module complements v1_formal_soak_scheduler.py. It is the only executor
that may produce qualifying evidence; --rehearsal always remains non-qualifying.
"""
from __future__ import annotations

import argparse, hashlib, importlib.util, json, os, pathlib, re, shutil, subprocess, sys, time, uuid
from collections import Counter
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("soak", ROOT / "scripts/v1_formal_soak_scheduler.py")
assert SPEC and SPEC.loader
soak = importlib.util.module_from_spec(SPEC); sys.modules[SPEC.name] = soak; SPEC.loader.exec_module(soak)
PH = re.compile(r"\{([A-Za-z][A-Za-z0-9]*)\}")
DISRUPTIVE = soak.DISRUPTIVE
REHEARSAL_MINIMA = {name: 1 for name in soak.MINIMA}


def run(argv: list[str], *, input_text: str | None = None) -> subprocess.CompletedProcess[str]:
    return subprocess.run(argv, text=True, input=input_text, capture_output=True)


def atomic_json(path: pathlib.Path, value: object) -> None:
    tmp = path.with_suffix(path.suffix + ".tmp")
    tmp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    os.replace(tmp, path)


def dir_stats(path: pathlib.Path) -> dict[str, int]:
    total = files = 0
    for item in path.rglob("*") if path.exists() else ():
        try:
            if item.is_file() and not item.is_symlink(): total += item.stat().st_size; files += 1
        except FileNotFoundError: pass
    return {"bytes": total, "files": files}


class Host:
    def __init__(self, cfg: dict[str, Any], cmap: dict[str, Any], out: pathlib.Path, rehearsal: bool):
        self.cfg, self.cmap, self.out, self.rehearsal = cfg, cmap, out, rehearsal
        self.unit = cmap["service"]["unit"]
        self.vars = {**cmap["variables"], **cfg.get("variables", {})}
        self.ops = cmap["operations"]
        self.generated = pathlib.Path(cfg["generatedRoot"]).resolve()
        self.paths = {k: pathlib.Path(v).resolve() for k, v in cfg["telemetryPaths"].items()}
        self.base_restarts = self.deliberate = 0
        self.not_ready_since: float | None = None
        self.archives: list[pathlib.Path] = []

    def props(self) -> dict[str, str]:
        cp = run(["systemctl","show",self.unit,"--property=ActiveState,MainPID,NRestarts,MemoryCurrent"])
        if cp.returncode: raise RuntimeError(cp.stderr.strip())
        return dict(line.split("=",1) for line in cp.stdout.splitlines() if "=" in line)

    def preflight(self) -> dict[str, Any]:
        if os.geteuid() != 0: raise RuntimeError("systemd runner requires root")
        if not self.rehearsal and os.environ.get("LINGONBERRY_FORMAL_SOAK_ACK") != soak.CANDIDATE:
            raise RuntimeError("formal acknowledgement mismatch")
        text = pathlib.Path("/etc/os-release").read_text()
        if 'ID=ubuntu' not in text or 'VERSION_ID="24.04"' not in text or os.uname().machine != "x86_64":
            raise RuntimeError("requires Ubuntu 24.04 x86_64")
        for tool in ("systemctl","curl","journalctl"):
            if not shutil.which(tool): raise RuntimeError(f"missing tool: {tool}")
        if self.cmap.get("candidateCommit") != soak.CANDIDATE: raise RuntimeError("command-map candidate mismatch")
        disabled = [k for k in soak.MINIMA if not self.ops.get(k,{}).get("enabled")]
        if disabled: raise RuntimeError(f"disabled required operations: {disabled}")
        for key, expected in (("storageBinary",soak.STORAGE_SHA256),("relayBinary",soak.RELAY_SHA256)):
            actual = hashlib.sha256(pathlib.Path(self.vars[key]).read_bytes()).hexdigest()
            if actual != expected: raise RuntimeError(f"{key} digest mismatch")
        for name,path in self.paths.items():
            if not path.is_absolute() or not path.exists() or path.is_symlink(): raise RuntimeError(f"unsafe telemetry path: {name}")
        p = self.props()
        if p.get("ActiveState") != "active": raise RuntimeError("service is not active")
        self.base_restarts = int(p.get("NRestarts") or 0)
        return {"unit":self.unit,"baselineNRestarts":self.base_restarts,"rehearsal":self.rehearsal}

    def telemetry(self, elapsed: int, phase: str) -> dict[str, Any]:
        p = self.props(); pid = int(p.get("MainPID") or 0)
        fds = 0
        try: fds = len(list(pathlib.Path(f"/proc/{pid}/fd").iterdir())) if pid else 0
        except FileNotFoundError: pass
        ready = run(["curl","-fsS",self.cmap["service"]["readyUrl"]]).returncode == 0
        now = time.monotonic()
        if ready: self.not_ready_since = None; not_ready = 0
        else: self.not_ready_since = self.not_ready_since or now; not_ready = int(now-self.not_ready_since)
        mi = {}
        for line in pathlib.Path('/proc/meminfo').read_text().splitlines():
            parts=line.replace(':','').split(); mi[parts[0]]=int(parts[1])*1024
        stat=os.statvfs(self.cfg["capacityPath"])
        jr=run(["journalctl","-u",self.unit,"--since",self.cfg["journalSince"],"--output=short-iso"])
        restarts=int(p.get("NRestarts") or 0)
        return {"timestamp":soak.utc_now(),"elapsedSeconds":elapsed,"samplePhase":phase,
          "serviceActive":p.get("ActiveState")=="active","ready":ready,"rssBytes":int(p.get("MemoryCurrent") or 0),
          "swapUsedBytes":max(0,mi.get('SwapTotal',0)-mi.get('SwapFree',0)),"fileDescriptors":fds,
          "freeDiskBytes":stat.f_bavail*stat.f_frsize,"freeInodes":stat.f_favail,
          "unexpectedRestarts":max(0,restarts-self.base_restarts-self.deliberate),"deliberateRestarts":self.deliberate,
          "readinessFailureSeconds":not_ready,"journalBytes":len(jr.stdout.encode())+len(jr.stderr.encode()),
          "paths":{k:dir_stats(v) for k,v in self.paths.items()}}

    def safe_path(self, name: str) -> pathlib.Path:
        self.generated.mkdir(parents=True,exist_ok=True)
        if self.generated.is_symlink(): raise RuntimeError("generatedRoot is a symlink")
        p=(self.generated/name).resolve()
        if p.parent != self.generated or p.exists() or p.is_symlink(): raise RuntimeError(f"unsafe generated path: {p}")
        return p

    def expand(self, token: str, extra: dict[str,str]) -> str:
        vals={**self.vars,**extra}
        def repl(m: re.Match[str]) -> str:
            if m.group(1) not in vals: raise RuntimeError(f"unknown placeholder: {m.group(1)}")
            return str(vals[m.group(1)])
        return PH.sub(repl,token)

    def execute(self, event: Any) -> dict[str, Any]:
        spec=self.ops[event.kind]; extra={}
        if event.kind=='backup': extra['generatedArchiveDir']=str(self.safe_path(f"backup-{event.ordinal:04d}"))
        if event.kind=='isolated_restore':
            if not self.archives: raise RuntimeError("restore scheduled before verified backup")
            extra['latestVerifiedArchiveDir']=str(self.archives[-1])
        if event.kind=='disk_pressure': extra['generatedDiskPressureDir']=str(self.safe_path(f"disk-pressure-{event.ordinal:04d}"))
        argv=[self.expand(x,extra) for x in spec['argv']]
        payload=None
        if spec['adapter']=='stdin-fixture':
            payload={'malformed':'{','oversized':'x'*int(self.cfg['oversizedInputBytes']),
                     'nested':'['*int(self.cfg['nestedInputDepth'])+'0'+']'*int(self.cfg['nestedInputDepth'])}[event.kind]
        cp=run(argv,input_text=payload); expected=spec.get('expectExitCodes',[0])
        if cp.returncode not in expected: raise RuntimeError(f"{event.kind} exit {cp.returncode}, expected {expected}: {cp.stderr[-500:]}")
        if event.kind in {'graceful_restart','abrupt_termination'}:
            self.deliberate += 1; deadline=time.monotonic()+int(self.cfg['restartRecoverySeconds'])
            while time.monotonic()<deadline:
                if run(['systemctl','is-active','--quiet',self.unit]).returncode==0 and run(['curl','-fsS',self.cmap['service']['readyUrl']]).returncode==0: break
                time.sleep(1)
            else: raise RuntimeError(f"{event.kind} recovery timeout")
        if event.kind=='backup':
            p=pathlib.Path(extra['generatedArchiveDir'])
            if not p.exists() or p.is_symlink(): raise RuntimeError("backup archive missing or symlinked")
            self.archives.append(p)
        return {"exitCode":cp.returncode,"stdoutSha256":hashlib.sha256(cp.stdout.encode()).hexdigest(),
                "stderrSha256":hashlib.sha256(cp.stderr.encode()).hexdigest(),"generated":extra}


def violation(m: dict[str,Any], t: Any, raw: dict[str,Any], baseline: dict[str,int]) -> str | None:
    v=soak.threshold_violation(m,t)
    if v: return v
    if raw.get('maximum_journal_bytes') is not None and m['journalBytes']>raw['maximum_journal_bytes']: return 'maximum journal bytes'
    for name,limit in raw.get('maximum_path_growth_bytes',{}).items():
        if name not in m['paths']: return f'missing telemetry path: {name}'
        if m['paths'][name]['bytes']-baseline.get(name,m['paths'][name]['bytes'])>limit: return f'maximum path growth: {name}'
    return None


def main() -> int:
    ap=argparse.ArgumentParser(); ap.add_argument('--out',type=pathlib.Path,required=True); ap.add_argument('--thresholds',type=pathlib.Path,required=True)
    ap.add_argument('--config',type=pathlib.Path,required=True); ap.add_argument('--command-map',type=pathlib.Path,required=True)
    ap.add_argument('--duration',type=int,default=soak.FORMAL_SECONDS); ap.add_argument('--tick-seconds',type=int,default=60)
    ap.add_argument('--rehearsal',action='store_true'); args=ap.parse_args()
    if args.out.exists(): raise RuntimeError('output already exists; runs are non-resumable')
    if not args.rehearsal and (args.duration<soak.FORMAL_SECONDS or args.tick_seconds!=60): raise RuntimeError('formal run requires >=72h and 60-second telemetry')
    if args.rehearsal and args.duration>=soak.FORMAL_SECONDS: raise RuntimeError('rehearsal must be shorter than formal run')
    args.out.mkdir(parents=True); run_id=f"v1-soak-{uuid.uuid4()}"; (args.out/'.run-id').write_text(run_id+'\n')
    ev=soak.Evidence(args.out,run_id); threshold_raw=json.loads(args.thresholds.read_text())
    base_keys=('minimum_free_disk_bytes','minimum_free_inodes','maximum_file_descriptors','maximum_rss_bytes','maximum_swap_used_bytes','maximum_readiness_failure_seconds','maximum_unexpected_restarts')
    limits=soak.Thresholds(**{k:threshold_raw[k] for k in base_keys})
    minima=REHEARSAL_MINIMA if args.rehearsal else soak.MINIMA; schedule=soak.build_schedule(args.duration,minima); soak.validate_distribution(schedule,args.duration,minima)
    host=Host(json.loads(args.config.read_text()),json.loads(args.command_map.read_text()),args.out,args.rehearsal)
    status='running'; reason=None; counts=Counter(); idx=0; baseline={}; started=soak.utc_now(); elapsed_done=0; pre={}
    try:
        pre=host.preflight()
        for src,name in ((args.config,'host-config.json'),(args.command_map,'command-map.json'),(args.thresholds,'thresholds.json')): shutil.copy2(src,args.out/'manifests'/name)
        atomic_json(args.out/'manifests'/'run.json',{'runId':run_id,'candidateCommit':soak.CANDIDATE,'rehearsal':args.rehearsal,'startedAt':started,'preflight':pre})
        atomic_json(args.out/'manifests'/'schedule.json',[vars(e) for e in schedule])
        wall=time.monotonic()
        for elapsed in range(0,args.duration+1,args.tick_seconds):
            elapsed_done=elapsed; m=host.telemetry(elapsed,'scheduled'); m['runId']=run_id; ev.metric(m)
            if not baseline: baseline={k:v['bytes'] for k,v in m['paths'].items()}
            bad=violation(m,limits,threshold_raw,baseline)
            if bad: raise RuntimeError(f'threshold violation: {bad}')
            while idx<len(schedule) and schedule[idx].offset<=elapsed:
                e=schedule[idx]
                if e.kind in DISRUPTIVE:
                    bm=host.telemetry(elapsed,f'pre-{e.kind}'); bm['runId']=run_id; ev.metric(bm)
                    bad=violation(bm,limits,threshold_raw,baseline)
                    if bad: raise RuntimeError(f'pre-operation threshold violation: {bad}')
                ev.event(soak.utc_now(),e.kind,'started',ordinal=e.ordinal,elapsedSeconds=elapsed)
                result=host.execute(e); counts[e.kind]+=1; idx+=1
                ev.event(soak.utc_now(),e.kind,'passed',ordinal=e.ordinal,elapsedSeconds=elapsed,result=result)
                if e.kind in DISRUPTIVE:
                    am=host.telemetry(elapsed,f'post-{e.kind}'); am['runId']=run_id; ev.metric(am)
                    bad=violation(am,limits,threshold_raw,baseline)
                    if bad: raise RuntimeError(f'post-operation threshold violation: {bad}')
                atomic_json(args.out/'partial'/'checkpoint.json',{'runId':run_id,'elapsedSeconds':elapsed,'nextEventIndex':idx,'counts':dict(counts),'status':'running'})
            if elapsed<args.duration: time.sleep(max(0,wall+elapsed+args.tick_seconds-time.monotonic()))
        if counts!=Counter(minima): raise RuntimeError(f'workload minima not met: {dict(counts)}')
        status='passed'
    except Exception as exc:
        status='failed'; reason=f'{type(exc).__name__}: {exc}'; ev.event(soak.utc_now(),'exception','failed',reason=reason)
    qualifying=bool(status=='passed' and not args.rehearsal and args.duration>=soak.FORMAL_SECONDS and args.tick_seconds==60 and counts==Counter(soak.MINIMA))
    atomic_json(args.out/'partial'/'checkpoint.json',{'runId':run_id,'elapsedSeconds':elapsed_done,'nextEventIndex':idx,'counts':dict(counts),'status':status,'stopReason':reason})
    ev.finalize({'schemaVersion':2,'runId':run_id,'candidateCommit':soak.CANDIDATE,'adapter':'systemd','rehearsal':args.rehearsal,
      'qualification':not args.rehearsal,'qualifyingPass':qualifying,'status':status,'stopReason':reason,'startedAt':started,'finishedAt':soak.utc_now(),
      'scheduledDurationSeconds':args.duration,'workloadCounts':dict(sorted(counts.items())),'workloadMinima':minima,'preflight':pre})
    return 0 if status=='passed' else 1

if __name__=='__main__': raise SystemExit(main())
