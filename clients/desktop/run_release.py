#!/usr/bin/env python3
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent


def main():
    subprocess.run(
        ["cargo", "build", "--release", "-p", "heroes-of-ironhold-desktop"],
        cwd=ROOT,
        check=True,
    )
    subprocess.run(
        ["cargo", "run", "--release", "-p", "heroes-of-ironhold-desktop"],
        cwd=ROOT,
        check=True,
    )


if __name__ == "__main__":
    main()
