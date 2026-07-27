#!/usr/bin/env python3
"""Active-candidate launcher for the v1 disk-pressure contract validator."""
from __future__ import annotations

import importlib.util
from pathlib import Path

ACTIVE_CANDIDATE = "8c6b48082205a3af555130eec1f3e7d2ac8811fe"


def load_implementation():
    path = Path(__file__).with_name("check-v1-disk-pressure-contract-legacy.py")
    spec = importlib.util.spec_from_file_location("lingonberry_disk_pressure_validator", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load disk-pressure validator: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.EXPECTED_CANDIDATE = ACTIVE_CANDIDATE
    return module


if __name__ == "__main__":
    implementation = load_implementation()
    implementation.main()
