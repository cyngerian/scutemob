# SR-9a — Integration-test consolidation

<!-- last_updated: 2026-07-14 -->

> **Read this before adding a test file under `crates/engine/tests/`.**
> The one-line rule: **never create a top-level `tests/*.rs`.** Put the file in a
> group directory and add its `mod` line to that group's `main.rs`.
> `tests/no_stray_test_binaries.rs` fails the suite if you don't.

## What was wrong

Cargo compiles and links every top-level `crates/engine/tests/*.rs` file as a
separate integration-test **binary**, each one statically linking the whole
engine. There were **297** of them. Nothing about the code was slow; the *shape
of the tree* was. A warm rebuild — touch `crates/engine/src/lib.rs`, then
`cargo test --all --no-run` — spent almost all of its time linking 297 copies of
the same library.

This is the `SR-9(a)` sub-item of `scutemob-61`, split out as `scutemob-69`.

## What changed

Every one of the 297 files moved, **verbatim**, into one of nine group
directories. Each group has a `main.rs` that does nothing but declare its
modules; Cargo picks up `tests/<group>/main.rs` as a single test target.

```
crates/engine/tests/
├── no_stray_test_binaries.rs   ← the gate (the only top-level file)
├── casting/{main.rs, casting.rs, mana_pool.rs, …}
├── combat/…
├── core/…
├── mechanics_a_d/…
├── mechanics_e_l/…
├── mechanics_m_z/…
├── primitives/…
├── rules/…
└── scripts/…
```

**297 binaries → 9.** (Plus the gate, which is deliberately its own tiny binary.)

| Target | Files | Lines | What belongs here |
|--------|------:|------:|-------------------|
| `core` | 26 | 12,965 | Engine foundations and the machine-checked invariant gates: state, turn, priority, resolution, SBAs, hashing, protocol, registry, deck validation |
| `rules` | 39 | 37,727 | Cross-cutting rules subsystems: layers, replacement, copy, triggers, targeting, protection, commander |
| `combat` | 6 | 6,267 | The combat system itself (combat *keywords* live in `mechanics_*`) |
| `casting` | 11 | 7,932 | Casting, mana, and cost payment |
| `primitives` | 38 | 32,165 | PB-* primitive batches — every `pb_*` / `primitive_*` file |
| `scripts` | 3 | 1,200 | The JSON game-script corpus and its replay harness |
| `mechanics_a_d` | 59 | 44,525 | Per-keyword / per-mechanic tests, names `a`–`d` |
| `mechanics_e_l` | 49 | 44,678 | Per-keyword / per-mechanic tests, names `e`–`l` |
| `mechanics_m_z` | 66 | 50,147 | Per-keyword / per-mechanic tests, names `m`–`z` |

The `mechanics_*` split is **alphabetical, not thematic**, and the letter
boundaries were chosen to balance line count (~45–50k each) so the nine targets
typecheck in parallel rather than behind one 140k-line straggler. There is no
meaning to `mechanics_a_d` beyond "a per-mechanic test whose name starts with
a–d". Do not look for one.

### Where a new test file goes

1. Testing a keyword or a single mechanic (the usual case) → `mechanics_<letter
   range matching the filename>/`.
2. Testing a PB-* primitive batch → `primitives/`.
3. Otherwise pick the subsystem group from the table.
4. **Add `mod <filename>;` to that group's `main.rs`.** Alphabetical order.

If a group ever needs to be added or renamed, update `EXPECTED_GROUPS` in
`tests/no_stray_test_binaries.rs` and the table above together — the gate
asserts the two agree with what is on disk.

### Running a subset

The target name is now the group, and the old per-file binary is a module path:

```bash
# whole group
cargo test -p mtg-engine --test rules

# one former binary
cargo test -p mtg-engine --test rules layers::

# one test
cargo test -p mtg-engine --test core state_hashing::hash_is_stable

# the golden-script corpus (was: --test run_all_scripts)
SCRIPT_FILTER=015_declare_attackers \
  cargo test -p mtg-engine --test scripts run_all_scripts -- --nocapture
```

Note the `::` in `layers::` — without it the filter is a substring match and will
also pick up `layer_correctness`.

## Measurements

Dev box (7800X3D, 16 threads), `CARGO_INCREMENTAL=0`, default dev profile.
Before-numbers come from a `git worktree` of the parent commit (`abe14f76`), so
both trees measure from their own freshly cold-built `target/`.

**Cold build** — `rm -rf target && cargo test --all --no-run`:

| | wall | `target/` |
|--|--:|--:|
| before (297 binaries) | 39.8 s | 19 GB |
| after (9 targets) | **24.0 s** | **2.2 GB** |

**Warm rebuild** — `touch crates/engine/src/lib.rs && cargo test --all --no-run`.
This is the number that matters: it is what every engine edit costs.

| | run 1 | run 2 | run 3 | median |
|--|--:|--:|--:|--:|
| before | 22.6 s | 34.2 s | 38.2 s | **34.2 s** |
| after | 15.0 s | 11.0 s | 11.1 s | **11.1 s** |

**≈3× faster.** But read the *shape* of the before-column, not just its median.
Each successive rebuild is slower than the last, reproducibly, and it is not
thermals: a warm rebuild of 297 test binaries rewrites ~18 GB of linked
executables to disk, and by the third run writeback has not caught up. The after
column climbs the other way — run 1 is the slowest, then it settles at ~11 s,
which is the normal shape of a page-cache warm-up. (An earlier measurement pass
in a `target/` that had accumulated 52 GB of stale artifacts read 43 / 53 / 60 s
before, and 11.0 / 10.9 / 10.9 s after — same conclusion, noisier.)

The disk figure is not cosmetic. CI already died once on it: `cargo test --all`
linking ~300 debuginfo-carrying test binaries produced a 68 GB `target/` and
overran the runner's disk, surfacing as `ld terminated with signal 7 [Bus error]`
with an LLVM "please file a bug" banner — the real cause, `No space left on
device`, was one line earlier (see the SR-1 gotcha in
`docs/sr-remediation-plan.md`). The `CARGO_PROFILE_{DEV,TEST}_DEBUG=0` workaround
in `.github/workflows/ci.yml` stays, but the pressure behind it is now ~9× lower.

**Test count is unchanged**: 3162 passing before, 3167 after — the five added
tests are the gate's own. 0 failed and 4 ignored, both before and after. Suite
count drops from 316 to 29, which is the whole point.

## The gate

`tests/no_stray_test_binaries.rs` is the only top-level test file, and it exists
to keep itself the only one. Eight tests (five original, three added by SR-18):

- `no_top_level_test_binaries` — any `tests/*.rs` other than the gate fails.
  Re-fragmentation happens one file at a time and is otherwise invisible.
- `every_expected_group_exists_and_has_a_module_root` — the group dirs on disk
  must exactly equal `EXPECTED_GROUPS`, and each must have a `main.rs`. A group
  dir without `main.rs` is silently not a Cargo target at all.
- `every_module_file_is_declared_in_its_group` — **this is the one that matters.**
  A `.rs` file inside a group dir with no `mod` line in that group's `main.rs`
  is not compiled. It does not error. It does not warn. Its tests simply cease
  to exist, and the suite goes green with less coverage than it had yesterday.
  That is the exact failure this consolidation could have introduced, so it is
  machine-checked in both directions (undeclared file, and declared-but-missing).
- `group_main_rs_declares_modules_and_nothing_else` — the check above is
  *textual*, and a textual check has holes: `#[cfg(feature = "never")] mod foo;`
  reads as declared and compiles to nothing; `#[path = "elsewhere.rs"] mod foo;`
  declares `foo` while never compiling `foo.rs`; an inline `mod foo { … }` does
  the same. Rather than teach the parser each attack, a group's `main.rs` may
  contain nothing but `//!` docs and bare `mod x;` lines. The grammar is small
  on purpose.
- `group_dirs_are_flat` — the declaration check reads one directory level, so a
  file at `tests/<group>/sub/foo.rs` would be invisible to it.
- `auto_built_targets_match_expected` (SR-18) — enumerates the targets Cargo will
  *actually* build (every top-level `*.rs`, plus every subdir with a `main.rs`)
  and pins that set to `EXPECTED_GROUPS + ALLOWED_TOP_LEVEL`. The group-existence
  check filters `NON_GROUP_DIRS` out before comparing; this admits no exemptions,
  so an exempted dir that grows a `main.rs` — a real, ungoverned target — fails
  here.
- `exempt_dirs_contain_no_rust_files` (SR-18) — a `.rs` dropped in an exempted dir
  (`proptest-regressions/`) is never compiled (the dir has no `main.rs`) and never
  seen by the group checks. Its tests silently do not exist. Forbidden.
- `no_module_level_cfg_in_group_files` (SR-18) — a module-level `#![cfg(...)]`
  inner attribute anywhere in a group *module* file compiles the enclosing module
  out, deleting every test in it, while the file stays present and `mod`-declared.
  `main.rs` content was already constrained; this constrains the module files. The
  detector strips comments and string/char literals and tokenises `# ! [ cfg` with
  arbitrary interior whitespace, so a block comment (`/* x */ #![cfg…]`) or a spaced
  form (`# ![cfg…]`) — both valid Rust that a first `split("//")` version missed,
  per the SR review — cannot hide it. `#![cfg_attr(…)]` matches too; non-deleting
  inner attributes (`#![allow(…)]`, `proptest!`'s `#![proptest_config(…)]`) do not.
  `module_cfg_detector_catches_obfuscations_and_spares_legit` pins both directions.

The last two of the original five exist because the first review of this change
went looking for ways to satisfy the gate while still doing the bad thing, and
found three. The three SR-18 additions come from a 2026-07-11 re-audit that found
three more the same way.

### Demonstrated, not assumed

Eight attacks were run against the gate before it was trusted. Read the middle
column: **the ordinary test suite goes green while coverage disappears.**

| Attack | Without the gate | With the gate |
|--------|------------------|---------------|
| Add a top-level `tests/stray_check.rs` | builds and passes; the tree is one file more fragmented and nothing said so | `no_top_level_test_binaries` fails, naming the file |
| Add `combat/melee_stub.rs` containing `assert!(false)`, no `mod` line | **`--test combat` → `ok. 75 passed; 0 failed`.** The failing test was never compiled. | `every_module_file_is_declared_in_its_group` fails, naming `melee_stub` |
| Delete `mod combat_harness;` from `combat/main.rs` | **`--test combat` → `ok. 69 passed; 0 failed`.** Six tests silently ceased to exist. | same test fails, naming `combat_harness` |
| `#[cfg(feature = "never")] mod combat_harness;` | **`--test combat` → `ok. 69 passed; 0 failed`.** Same six tests gone, and the line is still *there* to read. | `group_main_rs_declares_modules_and_nothing_else` fails on the attribute |
| `#[path = "combat.rs"] mod combat_harness;` | `combat_harness.rs` is declared and never compiled | same test fails on the attribute |
| `pub mod combat_harness;` | harmless, but the textual parser misread it as undeclared and failed the *wrong* test | same test fails with a message that names the actual problem |
| `mkdir combat/sub; combat/sub/foo.rs` | invisible to a one-level directory read | `group_dirs_are_flat` fails |
| Add `mod ghost;` with no `ghost.rs` | `rustc` catches this one on its own | gate also fails (`declares modules with no file`) |

Rows three and four are the whole reason this file exists. A `mod` line is one
easily-lost token, and losing it converts a test file into a text file — while
the suite reports success.

The last row is honest bookkeeping: `rustc` already rejects a phantom module, so
that half of the check is defensive rather than load-bearing. A demonstration
that stops at a compile error measures the compiler, not the gate.

### What the gate does not cover

Per the standing SR-track lesson (*the author verifies the gate fires on the
thing they were thinking about, and never enumerates the things it is not
pointed at*), the omissions, deliberately:

- **Other crates.** `tools/rust-analyzer-mcp/tests/` has its own binaries. It has
  one file; it is not the problem, and it is outside the engine.
- **Unit tests in `src/`.** `#[cfg(test)] mod tests` compiles into the library's
  own test binary and costs one link total. Untouched, unwatched.
- **`benches/`.** One target, `harness = false`. Untouched.
- **`[[test]]` sections in `Cargo.toml`.** Cargo's `autotests` discovery is what
  the gate models. Hand-declaring a test target bypasses it entirely. Nobody has,
  and the manifest is short enough to read.
- **Per-test `#[cfg]` / deletions *inside* a module file.** A `#[cfg(…)]` on a
  single `#[test]` within `combat_harness.rs`, or a `#[test]` deleted outright, is
  out of scope for a structural gate — that is what review and the test count are
  for. SR-18 does cover the whole-module case (`no_module_level_cfg_in_group_files`
  catches a module-level `#![cfg(...)]`, which would delete *every* test in the
  file at once); the residue left uncovered is the per-item form.
- **Group *membership*.** Nothing stops someone dropping a keyword test into
  `core/`. The gate checks that every file is compiled, not that it is filed
  sensibly. A taste question, left to review.
- **Symlinks.** `is_dir()` follows them, so a symlinked group dir would pass.
  There is no plausible reason for one to exist here.

## Two warts, deliberately left

1. `scripts/run_all_scripts.rs` pulls in `scripts/script_replay.rs` with
   `include!` rather than as a sibling module. Inside a single binary that is now
   pointless duplication: `script_replay.rs` is compiled twice into `scripts`,
   and its four unit tests run twice. Removing the `include!` in favour of
   `use super::script_replay::…` would drop the duplicate compile *and* the four
   duplicate test runs — but the latter would change the test count, and SR-9a's
   acceptance criterion is that the count does not move. Left for a follow-up.
   The `#[allow(dead_code)]` on `AssertionMismatch` is the visible residue: its
   fields are read only through the `include!` copy.
2. Three `include_str!` paths gained a `../` (`keyword_registry.rs`,
   `pending_trigger_shape.rs`, `pb_ac9_wheel_and_misc.rs`). `include_str!` is
   resolved relative to *the file containing it*, not the manifest — so a file
   that moves one level deeper must be edited. `env!("CARGO_MANIFEST_DIR")` paths
   and the runtime `Path::new("../../test-data/…")` in `run_all_scripts.rs` are
   manifest- and cwd-relative respectively, and did not move.

## Not in scope

SR-9 had three sub-items. This is only the first.

- **SR-9b** (`scutemob-70`) — harness-vs-direct-dispatch equivalence property
  test.
- **SR-9c** (`scutemob-71`) — golden-script corpus triage; 176 of 271 scripts are
  `pending_review` and silently skipped.
