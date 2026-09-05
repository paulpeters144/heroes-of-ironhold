# Description

Break the "Rules" section currently living in `AGENTS.md` into logical, standalone `.md` files, and wire them into opencode via the `instructions` field in a new `opencode.json`. `AGENTS.md` keeps its project info (Structure, Assets, Build & Run, Docs) but slimmed down to reference the new rule files instead of embedding the rules inline.

# Objects

## rules/
A new directory at the repo root containing the split rule files. Grouping is by concern; each file holds the full, unmodified text of its current rule(s) plus surrounding context.

- `rules/store-access.md` — the fluent store accessor-chain rule (the `first`/`get_child` example and the key-method reference list)
- `rules/conventions.md` — the coding-convention rules: `sys_*.rs` naming, `ctx.dt` (never `get_frame_time()`), assets extracted at construction (never `Assets` accessors in `update()`/`draw()`)
- `rules/ownership.md` — never modify `pico_entity_store` or `src/util/estore.rs` unless explicitly told to
- `rules/git.md` — never commit changes unless the user explicitly asks

## opencode.json
New project config at repo root declaring the rule files so opencode loads them alongside `AGENTS.md`.

- `$schema: string` — `"https://opencode.ai/config.json"`
- `instructions: string[]` — paths/globs of the rule files, e.g. `["rules/*.md"]` (or explicit per-file paths)

## AGENTS.md
Existing file; the `## Rules` section is removed and its content replaced by pointers/references to the new `rules/` files (e.g. a short `## Rules` section listing each file and when it applies).

# Services

None.

# TODO

- [ ] Create `rules/` directory and the four rule `.md` files
- [ ] Move each AGENTS.md rule into its logical file, unmodified
- [ ] Create `opencode.json` with `instructions` pointing at `rules/*.md`
- [ ] Replace the AGENTS.md `## Rules` section with references to the rule files
- [ ] Verify the rule files load correctly in a fresh opencode session

# TODO Explanation

## Create `rules/` directory and the four rule `.md` files
Create `rules/store-access.md`, `rules/conventions.md`, `rules/ownership.md`, and `rules/git.md` under a new root-level `rules/` directory. Each file is plain markdown describing its rules so it reads naturally as an instruction source.

## Move each AGENTS.md rule into its logical file, unmodified
Copy the existing rules from `AGENTS.md` `## Rules` verbatim into their new homes on lines matching the grouping above. No rewording or trimming in this pass — the store-access rule keeps its full example and method reference list.

## Create `opencode.json` with `instructions` pointing at `rules/*.md`
Add the `opencode.json` object (with `$schema` and an `instructions` array targeting the new files). This is required because opencode does not parse file references inside `AGENTS.md` — rule files only load via the `instructions` field. Use `"rules/*.md"` so future rule files are picked up automatically, or list explicit per-file paths if glob loading is not desired.

## Replace the AGENTS.md `## Rules` section with references to the rule files
Keep `AGENTS.md` as the project's entry point under opencode's auto-load, but shrink `## Rules` to a short index pointing at `rules/` — e.g. one line per rule file saying when it applies. Optionally include the AGENTS.md `@path` reference convention so agent sessions know rules are split across files.

## Verify the rule files load correctly in a fresh opencode session
After config changes, confirm opencode picks up all files. Note opencode loads config at startup and does not hot-reload, so a restart is required before verification.

# Open Questions

- Should the existing store-access rule (the long one with the example and method list) be further subdivided, or kept as one file?
- Does the user prefer one `instructions` entry per file in `opencode.json`, or a single glob like `"rules/*.md"`?
- Where should the rule files live — a top-level `rules/` directory, `docs/`, or `.opencode/`?
- Should AGENTS.md keep a full `## Rules` index with explanations, or just bare one-line pointers to each file?
- Should any existing non-Rules content in AGENTS.md (Project Structure, Assets, Build & Run, Docs) also be split out, or is this strictly the Rules section?

# Out of Scope

- Rewording, adding, or removing the actual rule content — this plan only relocates rules
- Splitting/providing non-Rules AGENTS.md sections into files
- Global (`~/.config/opencode/`) rule files — these are project-specific
- Changing the workflow behind any rule (e.g. the store-access patterns themselves)