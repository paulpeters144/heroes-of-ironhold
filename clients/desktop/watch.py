#!/usr/bin/env python3
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

WATCH_DIRS = [
    ROOT / "src",
    ROOT / "clients",
    ROOT / "crates",
    ROOT / "assets",
]
WATCH_FILES = [
    ROOT / "Cargo.toml",
    ROOT / "Cargo.lock",
]
IGNORE_PARTS = {"target", ".git", "node_modules"}
POLL_INTERVAL = 1.0
DEBOUNCE_INTERVAL = 1.0


def snapshot():
    state = {}
    for path in WATCH_FILES:
        if path.is_file():
            stat = path.stat()
            state[str(path)] = (stat.st_mtime_ns, stat.st_size)
    for directory in WATCH_DIRS:
        if not directory.is_dir():
            continue
        for path in directory.rglob("*"):
            if not path.is_file():
                continue
            if any(part in IGNORE_PARTS for part in path.parts):
                continue
            stat = path.stat()
            state[str(path)] = (stat.st_mtime_ns, stat.st_size)
    return state


def main():
    process = None

    def stop():
        nonlocal process
        if process is not None and process.poll() is None:
            process.terminate()
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait()
        process = None

    def start():
        nonlocal process
        process = subprocess.Popen(
            ["cargo", "run", "-p", "heroes-of-ironhold-desktop"],
            cwd=ROOT,
        )

    def handle_sigint(signum, frame):
        stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, handle_sigint)

    previous = snapshot()
    print("watching for changes (ctrl+c to quit)...")
    start()

    last_change = 0.0

    while True:
        time.sleep(POLL_INTERVAL)
        if process is not None and process.poll() is not None:
            print(f"process exited (code {process.returncode}), waiting for changes...")
            process = None

        current = snapshot()
        if current == previous:
            continue

        previous = current
        now = time.monotonic()
        if now - last_change < DEBOUNCE_INTERVAL:
            continue
        last_change = now

        print("change detected, restarting...")
        stop()
        start()


if __name__ == "__main__":
    main()
