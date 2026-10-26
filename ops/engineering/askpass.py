#!/usr/bin/env python3
"""Git receives the token through stdin helper output, never a URL or argv."""
import os
import sys

print("x-access-token" if "username" in sys.argv[-1].lower() else os.environ["GH_TOKEN"])
