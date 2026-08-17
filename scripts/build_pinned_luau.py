"""Build the official Luau tools at the revision used by compiler CI."""

from __future__ import annotations

import subprocess
from pathlib import Path


LUAU_REVISION = "af6afddc651f3e8a272b1742d7f56695f9a9a278"
LUAU_REPOSITORY = "https://github.com/luau-lang/luau.git"


def run(command: list[str], cwd: Path | None = None) -> None:
    """Run one bootstrap command and stop on the first failure."""
    subprocess.run(command, cwd=cwd, check=True)


def main() -> None:
    """Clone, pin, and build the three official Luau executables."""
    repository_root = Path(__file__).resolve().parents[1]
    checkout = repository_root / "references" / "checkouts" / "luau"
    checkout.parent.mkdir(parents=True, exist_ok=True)
    if not checkout.exists():
        run(["git", "clone", "--filter=blob:none", LUAU_REPOSITORY, str(checkout)])
    run(["git", "-C", str(checkout), "fetch", "--depth", "1", "origin", LUAU_REVISION])
    run(["git", "-C", str(checkout), "checkout", "--detach", LUAU_REVISION])
    completed_revision = subprocess.run(
        ["git", "-C", str(checkout), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if completed_revision != LUAU_REVISION:
        raise RuntimeError(
            f"Luau checkout is {completed_revision}, expected {LUAU_REVISION}"
        )
    run(
        ["make", "config=release", "luau", "luau-analyze", "luau-compile"],
        cwd=checkout,
    )


if __name__ == "__main__":
    main()
