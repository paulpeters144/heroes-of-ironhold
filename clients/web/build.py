#!/usr/bin/env python3
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
HERE = Path(__file__).resolve().parent
DIST = HERE / "dist"
ASSETS_SRC = ROOT / "assets"
ASSETS_DST = DIST / "assets"
WASM_SRC = ROOT / "target" / "wasm32-unknown-unknown" / "release" / "heroes-of-ironhold-web.wasm"
WASM_NAME = "heroes-of-ironhold-web.wasm"


def main():
    subprocess.run(
        ["cargo", "build", "-p", "heroes-of-ironhold-web", "--target", "wasm32-unknown-unknown", "--release"],
        cwd=ROOT,
        check=True,
    )

    DIST.mkdir(parents=True, exist_ok=True)
    shutil.copy2(WASM_SRC, DIST / WASM_NAME)
    shutil.copy2(HERE / "index.html", DIST / "index.html")

    if ASSETS_DST.exists():
        shutil.rmtree(ASSETS_DST)
    shutil.copytree(ASSETS_SRC, ASSETS_DST)

    print("Build complete. Files in clients/web/dist/")
    print("Serve with: python3 clients/web/serve.py")


if __name__ == "__main__":
    main()
