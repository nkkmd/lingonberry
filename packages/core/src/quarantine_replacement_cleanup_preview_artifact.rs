use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use lingonberry_protocol::to_canonical_json;

use crate::{
    quarantine_replacement_cleanup_plan_json, quarantine_replacement_cleanup_proof_json,
    store_error, QuarantineReplacementCleanupPlan, QuarantineReplacementCleanupProof, StoreError,
};

pub const QUARANTINE_REPLACEMENT_CLEANUP_PLAN_FILE: &str =
    "quarantine-replacement-cleanup-plan.json";
pub const QUARANTINE_REPLACEMENT_CLEANUP_PLAN_DIGEST_FILE: &str =
    "quarantine-replacement-cleanup-plan.digest";
pub const QUARANTINE_REPLACEMENT_CLEANUP_PROOF_FILE: &str =
    "quarantine-replacement-cleanup-proof.json";
pub const QUARANTINE_REPLACEMENT_CLEANUP_PROOF_DIGEST_FILE: &str =
    "quarantine-replacement-cleanup-proof.digest";

struct ArtifactPairNames<'a> {
    json: &'a str,
    digest: &'a str,
    json_tmp: &'a str,
    digest_tmp: &'a str,
}

const PLAN_ARTIFACTS: ArtifactPairNames<'static> = ArtifactPairNames {
    json: QUARANTINE_REPLACEMENT_CLEANUP_PLAN_FILE,
    digest: QUARANTINE_REPLACEMENT_CLEANUP_PLAN_DIGEST_FILE,
    json_tmp: ".quarantine-replacement-cleanup-plan.json.tmp",
    digest_tmp: ".quarantine-replacement-cleanup-plan.digest.tmp",
};

const PROOF_ARTIFACTS: ArtifactPairNames<'static> = ArtifactPairNames {
    json: QUARANTINE_REPLACEMENT_CLEANUP_PROOF_FILE,
    digest: QUARANTINE_REPLACEMENT_CLEANUP_PROOF_DIGEST_FILE,
    json_tmp: ".quarantine-replacement-cleanup-proof.json.tmp",
    digest_tmp: ".quarantine-replacement-cleanup-proof.digest.tmp",
};

pub fn publish_quarantine_replacement_cleanup_preview_artifacts(
    output_dir: impl AsRef<Path>,
    plan: &QuarantineReplacementCleanupPlan,
) -> Result<QuarantineReplacementCleanupProof, StoreError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir).map_err(io_error)?;

    let plan_text = to_canonical_json(&quarantine_replacement_cleanup_plan_json(plan));
    let plan_digest = integrity_digest(plan_text.as_bytes());
    let proof = QuarantineReplacementCleanupProof {
        plan: plan.clone(),
        plan_digest: plan_digest.clone(),
    };
    let proof_text = to_canonical_json(&quarantine_replacement_cleanup_proof_json(&proof));
    let proof_digest = integrity_digest(proof_text.as_bytes());

    publish_pair(output_dir, &PLAN_ARTIFACTS, &plan_text, &plan_digest)?;
    publish_pair(output_dir, &PROOF_ARTIFACTS, &proof_text, &proof_digest)?;
    sync_directory(output_dir)?;
    verify_quarantine_replacement_cleanup_preview_artifacts(output_dir, &proof)?;
    Ok(proof)
}

pub fn verify_quarantine_replacement_cleanup_preview_artifacts(
    output_dir: impl AsRef<Path>,
    expected: &QuarantineReplacementCleanupProof,
) -> Result<(), StoreError> {
    let output_dir = output_dir.as_ref();
    let expected_plan = to_canonical_json(&quarantine_replacement_cleanup_plan_json(&expected.plan));
    let expected_plan_digest = integrity_digest(expected_plan.as_bytes());
    if expected.plan_digest != expected_plan_digest {
        return Err(artifact_error(
            "proof plan digest does not match canonical plan",
        ));
    }
    let expected_proof = to_canonical_json(&quarantine_replacement_cleanup_proof_json(expected));
    let expected_proof_digest = integrity_digest(expected_proof.as_bytes());

    verify_pair(
        output_dir,
        &PLAN_ARTIFACTS,
        &expected_plan,
        &expected_plan_digest,
    )?;
    verify_pair(
        output_dir,
        &PROOF_ARTIFACTS,
        &expected_proof,
        &expected_proof_digest,
    )
}

fn publish_pair(
    dir: &Path,
    names: &ArtifactPairNames<'_>,
    text: &str,
    digest: &str,
) -> Result<(), StoreError> {
    let json_path = dir.join(names.json);
    let digest_path = dir.join(names.digest);
    if json_path.exists() || digest_path.exists() {
        return verify_pair(dir, names, text, digest);
    }

    let json_tmp = dir.join(names.json_tmp);
    let digest_tmp = dir.join(names.digest_tmp);
    if json_tmp.exists() || digest_tmp.exists() {
        return Err(artifact_error(
            "stale cleanup preview temporary artifact requires manual review",
        ));
    }

    let result = (|| {
        write_new_synced(&json_tmp, text.as_bytes())?;
        write_new_synced(&digest_tmp, format!("{digest}\n").as_bytes())?;
        fs::rename(&json_tmp, &json_path).map_err(io_error)?;
        fs::rename(&digest_tmp, &digest_path).map_err(io_error)?;
        sync_directory(dir)
    })();
    if result.is_err() {
        let _ = fs::remove_file(json_tmp);
        let _ = fs::remove_file(digest_tmp);
    }
    result
}

fn verify_pair(
    dir: &Path,
    names: &ArtifactPairNames<'_>,
    expected_text: &str,
    expected_digest: &str,
) -> Result<(), StoreError> {
    let json_path = dir.join(names.json);
    let digest_path = dir.join(names.digest);
    if !json_path.is_file() || !digest_path.is_file() {
        return Err(artifact_error(
            "cleanup preview artifact pair is incomplete",
        ));
    }
    let actual_text = fs::read_to_string(json_path).map_err(io_error)?;
    let actual_digest = fs::read_to_string(digest_path).map_err(io_error)?;
    if actual_text != expected_text
        || actual_digest.trim() != expected_digest
        || integrity_digest(actual_text.as_bytes()) != expected_digest
    {
        return Err(artifact_error("cleanup preview artifact mismatch"));
    }
    Ok(())
}

fn write_new_synced(path: &Path, bytes: &[u8]) -> Result<(), StoreError> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}

fn sync_directory(path: &Path) -> Result<(), StoreError> {
    File::open(path)
        .map_err(io_error)?
        .sync_all()
        .map_err(io_error)
}

fn integrity_digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn io_error(error: std::io::Error) -> StoreError {
    artifact_error(error.to_string())
}

fn artifact_error(message: impl Into<String>) -> StoreError {
    store_error(
        "LB_QUARANTINE_REPLACEMENT_CLEANUP_PREVIEW_ARTIFACT",
        message,
    )
}
