#!/usr/bin/env python3
"""Active-candidate launcher for the Lingonberry v1.0 formal soak scheduler.

The implementation is retained in ``v1_formal_soak_scheduler_legacy.py`` so the
reviewed scheduling and evidence logic remains unchanged. This launcher fixes
all candidate-bound identities before delegating to that implementation.
"""
from __future__ import annotations

import importlib.util
from pathlib import Path

CANDIDATE = "8c6b48082205a3af555130eec1f3e7d2ac8811fe"
STORAGE_SHA256 = "737b148de48bc2ed2f96b3fb8e068e4c696f73d4069e7eaf89b76eaa6a610507"
RELAY_SHA256 = "23b5cd4044b69a483a457a71164ac5376370793bd502518e2e7d1baeab34a81c"


def load_implementation():
    path = Path(__file__).with_name("v1_formal_soak_scheduler_legacy.py")
    spec = importlib.util.spec_from_file_location("lingonberry_v1_formal_soak_impl", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load formal soak implementation: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.CANDIDATE = CANDIDATE
    module.STORAGE_SHA256 = STORAGE_SHA256
    module.RELAY_SHA256 = RELAY_SHA256
    return module


if __name__ == "__main__":
    implementation = load_implementation()
    raise SystemExit(implementation.main())
