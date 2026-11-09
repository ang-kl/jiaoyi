#!/usr/bin/env python3
"""Repository CLI for the deterministic monitor packaged in the engineering image."""
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "engineering"))
from monitoring.runtime import main

if __name__ == "__main__":
    main()
