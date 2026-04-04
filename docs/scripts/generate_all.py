#!/usr/bin/env python3
"""Generate auto-generated sphinx-needs files for herkos traceability."""

import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).parent


def main():
    scripts = [
        SCRIPTS_DIR / "generate_wasm_spec_needs.py",
        SCRIPTS_DIR / "generate_test_needs.py",
        SCRIPTS_DIR / "generate_impl_needs.py",
    ]

    for script in scripts:
        if script.exists():
            print(f"Running {script.name}...")
            subprocess.check_call([sys.executable, str(script)])
        else:
            print(f"Skipping {script.name} (not yet created)")


if __name__ == "__main__":
    main()
