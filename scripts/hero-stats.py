#!/usr/bin/env python3
"""Print a stat table for a hero class across a level range.

Usage:
    python scripts/hero-stats.py <hero> --lvl=<start>:<end>
    python scripts/hero-stats.py <hero> --lvl=<level>

Examples:
    python scripts/hero-stats.py knight --lvl=1:60
    python scripts/hero-stats.py knight --lvl=60
    python scripts/hero-stats.py hunter --lvl=1:13
"""

import json
import os
import sys
from decimal import Decimal, ROUND_HALF_UP

HEROES_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "heroes.json")

STAT_KEYS = ["hp", "mp", "str", "arm", "int", "res", "eva", "acc", "crit"]
STAT_HEADERS = ["HP", "MP", "STR", "ARM", "INT", "RES", "EVA", "ACC", "CRIT"]


def calc_stat(base, mod, level: int) -> int:
    value = Decimal(str(base)) * (Decimal(1) + Decimal(str(mod)) * (level - 1))
    return int(value.to_integral_value(rounding=ROUND_HALF_UP))


def load_heroes(path: str) -> dict:
    with open(path) as f:
        return json.load(f)


def parse_level_range(arg: str) -> tuple[int, int]:
    if not arg.startswith("--lvl="):
        raise ValueError(f"Bad level arg: {arg}")
    rest = arg[len("--lvl="):]
    parts = rest.split(":")
    if len(parts) == 1:
        lvl = int(parts[0])
        return lvl, lvl
    if len(parts) == 2:
        return int(parts[0]), int(parts[1])
    raise ValueError(f"Expected --lvl=<level> or --lvl=<start>:<end>, got --lvl={rest}")


def print_table(hero_name: str, hero: dict, start: int, end: int):
    base = hero["base"]
    mods = hero["mods"]

    col_widths = [5] + [max(len(h), 6) for h in STAT_HEADERS]
    header = "Level".ljust(col_widths[0]) + "".join(
        STAT_HEADERS[i].rjust(col_widths[i + 1]) for i in range(len(STAT_HEADERS))
    )
    sep = "-" * len(header)

    label = f"level {start}" if start == end else f"levels {start}-{end}"
    print(f"\n  {hero_name.upper()}  ({label})\n")
    print(header)
    print(sep)

    for lvl in range(start, end + 1):
        row = str(lvl).ljust(col_widths[0])
        for i, key in enumerate(STAT_KEYS):
            val = calc_stat(base[key], mods[key], lvl)
            row += str(val).rjust(col_widths[i + 1])
        print(row)

    print()


def main():
    args = sys.argv[1:]
    if not args or args[0] in ("-h", "--help"):
        print(__doc__.strip())
        sys.exit(0)

    hero_name = args[0].lower()
    level_arg = None
    for a in args[1:]:
        if a.startswith("--lvl="):
            level_arg = a
            break

    if level_arg is None:
        print("Error: --lvl=<start>:<end> is required", file=sys.stderr)
        sys.exit(1)

    heroes = load_heroes(HEROES_PATH)

    if hero_name not in heroes:
        available = ", ".join(sorted(heroes.keys()))
        print(f"Error: unknown hero '{hero_name}'. Available: {available}", file=sys.stderr)
        sys.exit(1)

    try:
        start, end = parse_level_range(level_arg)
    except ValueError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

    if start < 1 or end < start:
        print(f"Error: invalid range {start}-{end}", file=sys.stderr)
        sys.exit(1)

    print_table(hero_name, heroes[hero_name], start, end)


if __name__ == "__main__":
    main()
