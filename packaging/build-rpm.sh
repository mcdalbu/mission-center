#!/usr/bin/bash
set -euo pipefail
repo_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
cd "$repo_root"
/usr/bin/python3 packaging/prepare-rpm.py
rpmbuild -ba --define "_topdir $repo_root/_rpm" packaging/mission-center-custom.spec
