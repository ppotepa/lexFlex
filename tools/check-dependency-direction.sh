#!/usr/bin/env bash
set -euo pipefail

cargo metadata \
  --format-version 1 \
  --no-deps \
| python3 -c '
import json
import sys

data = json.load(sys.stdin)

deps = {
    package["name"]: {
        dependency["name"]
        for dependency in package["dependencies"]
    }
    for package in data["packages"]
}

rules = {
    "lexflex-model": {
        "lexflex-lingua",
        "lexflex-language",
        "lexflex-engine",
    },
    "lexflex-lingua": {
        "lexflex-language",
        "lexflex-engine",
    },
    "lexflex-language": {
        "lexflex-engine",
    },
}

failed = False

for package, forbidden in rules.items():
    violations = sorted(deps.get(package, set()) & forbidden)
    if violations:
        failed = True
        print(f"{package}: {violations}")

if failed:
    raise SystemExit(1)
'
