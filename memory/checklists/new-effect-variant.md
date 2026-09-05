# Checklist — adding a variant to a core engine enum

<!-- last_updated: 2026-09-05 -->

**Scope**: a new variant of `Effect`, `AbilityDefinition`, `KeywordAbility` or `StackObjectKind`.
Adopted 2026-09-05 (LL-3, `scutemob-257`) from `docs/mtg-engine-landscape-assessment.md` §9 row a,
modelled on the phase.rs clone's `.claude/skills/add-engine-effect/SKILL.md` (MIT/Apache-2.0), whose own summary of
why a checklist exists is the reason this one does: *"Missing any step causes silent failures —
effects parse but don't resolve, resolve but don't target, target but don't animate."*

**Read this before you propose the variant, not after you write it.** Half of these points fail
**silently** — the workspace builds, the suite is green, and the card is quietly wrong at a table.
The compile-error points are the cheap ones.

**Before anything below**: `memory/conventions.md` → "Parameterize, don't proliferate". The
cheapest registration is the one you do not have to do. A leaf-level parameterization of an
existing variant costs zero rows on this page; a sibling costs all of them.

## How to read a row

| Column | Meaning |
|---|---|
| **Where** | The file. Paths, not line numbers — line numbers rot between batches. |
| **Symbol** | What to grep for inside that file to land on the site. |
| **What to add** | The edit. |
| **If you miss it** | **COMPILE** = rustc or a gate stops you. **SILENT** = it builds green and the behaviour is wrong. |

Verify this page against HEAD before trusting a row:

```bash
python3 - memory/checklists/new-effect-variant.md <<'PY'
import os, re, sys
ok = bad = 0
for ln, line in enumerate(open(sys.argv[1]), 1):
    m = re.match(r'\s*\|\s*`([^`]+)`\s*\|\s*(?:`([^`]+)`|—)\s*\|', line)
    if not m:
        continue
    path, sym = m.group(1), m.group(2)
    if not os.path.exists(path):
        print(f'L{ln}: STALE PATH {path}'); bad += 1
    elif sym and os.path.isfile(path) and sym not in open(path, errors='replace').read():
        print(f'L{ln}: SYMBOL NOT FOUND `{sym}` in {path}'); bad += 1
    else:
        ok += 1
print(f'{ok} rows verified, {bad} stale')
PY
```

A stale row is a defect in this file, not a reason to skip the step: find where the site moved,
fix the row in the same batch, and say so in the notes file.
## The four enums, and where they live

| Defined in | Symbol | Variants at HEAD (`9677fa0c`) |
|---|---|---|
| `crates/card-types/src/cards/card_definition.rs` | `pub enum Effect` | 106 |
| `crates/card-types/src/cards/card_definition.rs` | `pub enum AbilityDefinition` | 68 |
| `crates/card-types/src/state/types.rs` | `pub enum KeywordAbility` | 166 |
| `crates/card-types/src/state/stack.rs` | `pub enum StackObjectKind` | 27 |

Counts drift. Re-derive them; do not quote this table at anyone.

## Phase 0 — the type

| Where | Symbol | What to add | If you miss it |
|---|---|---|---|
| `crates/card-types/src/cards/card_definition.rs` | `pub enum Effect` | the variant, with typed fields — an enum, never a `bool` flag, for a semantic distinction | n/a (this is the change) |
| `crates/card-types/src/cards/helpers.rs` | `pub use` | **nothing, for a variant.** `Effect`, `AbilityDefinition` and `KeywordAbility` are already re-exported (`StackObjectKind` deliberately is not — a card-def author never constructs one). Add here only if the variant's payload names a **new type** | **COMPILE** — card defs fail with "undeclared type", and only in `card-defs`, which the engine's own build does not exercise |

`crates/card-defs` depends on `card-types` only, never on the engine (SR-6). If your variant's
payload needs a helper, it goes in `card-types`, not in the engine.

## Phase 1 — behaviour (the compile-error points)

These are exhaustive, wildcard-free matches. rustc stops you.

| Where | Symbol | What to add | If you miss it |
|---|---|---|---|
| `crates/engine/src/effects/mod.rs` | `execute_effect_inner` | the `Effect` arm that actually executes it | **COMPILE** — and per split-on-touch (`memory/conventions.md`), move the arm you are editing into its own module first |
| `crates/engine/src/rules/resolution.rs` | `resolve_top_of_stack_inner` | the `StackObjectKind` arm: what happens when this stack object resolves (CR 608.1) | **COMPILE** |
| `crates/engine/src/state/keyword_registry.rs` | `pub fn handling` | classify the `KeywordAbility` as `Handled { sites }` or `Marker { carrier, cr }` (SR-5) | **COMPILE**, then `crates/engine/tests/core/keyword_registry.rs` checks the claim against a comment-stripped source scan — a declared site that does not exist, or a real site not declared, both redden |
| `crates/engine/src/state/ability_definition_registry.rs` | `pub fn handling` | classify the `AbilityDefinition` the same way | **COMPILE.** This is the ONLY exhaustive dispatch over the whole enum: its own module doc says that without it "a newly added variant compiles everywhere and is silently inert" |
| `crates/engine/src/state/stack_registry.rs` | `card_in_stack_zone` | does this `StackObjectKind` own a card sitting in `ZoneId::Stack`? (CR 701.6a) | **COMPILE.** Deliberately wildcard-free: `Effect::CounterSpell` guessed this from the variant's NAME before PB-DX25 and no-opped on `MutatingCreatureSpell` |
| `crates/engine/src/state/stack_registry.rs` | `source_of` | the source object for the new kind | **COMPILE** |
| `crates/simulator/src/invariants.rs` | `stack_card_of` | the same `StackObjectKind` answer, **deliberately duplicated** | **COMPILE.** Do not delegate it to `stack_registry`: this check exists to catch the engine getting the classification wrong, and reading the engine's own answer back would make it agree with the bug |

## Phase 2 — hashing (SR-8, compile error, then correctness)

`crates/engine/src/state/hash.rs` carries one wildcard-free `HashInto` impl per enum. Each arm
writes an **explicit `u8` discriminant**: append the next unused id, never renumber an existing one.

| Where | Symbol | If you miss it |
|---|---|---|
| `crates/engine/src/state/hash.rs` | `impl HashInto for Effect` | **COMPILE** |
| `crates/engine/src/state/hash.rs` | `impl HashInto for AbilityDefinition` | **COMPILE** |
| `crates/engine/src/state/hash.rs` | `impl HashInto for KeywordAbility` | **COMPILE** |
| `crates/engine/src/state/hash.rs` | `impl HashInto for StackObjectKind` | **COMPILE** |

Compiling is not the same as hashing correctly. **Adding the arm but not hashing one of its
payload fields is SILENT**: two different states hash equal, and undo / replay / the SR-9b
fingerprint diverge with no error at the site that caused it.

## Phase 3 — display and the rosters (the silent ones)

| Where | Symbol | What to add | If you miss it |
|---|---|---|---|
| `crates/view-model/src/lib.rs` | `stack_kind_info` | the `StackObjectKind` label + source id — the single shared classification both clients render through | **COMPILE** (exhaustive) |
| `crates/view-model/src/lib.rs` | `format_keyword` | the `KeywordAbility` display string | **COMPILE** (exhaustive) — and runners miss this one about half the time, so `cargo build --workspace` after every impl phase, not at the end |
| `tools/tui/src/play/panels/stack_view.rs` | `StackObjectKind` | the TUI stack-panel label | **COMPILE** (exhaustive) |
| `crates/engine/src/state/keyword_registry.rs` | `pub fn all_keywords` | one representative value of the new variant | **TEST** — `all_keywords_covers_every_variant` parses `pub enum KeywordAbility` out of the source and set-compares both directions, so it reddens loudly. Nothing in the engine does |
| `crates/engine/src/state/ability_definition_registry.rs` | `pub fn all_ability_definitions` | same, for `AbilityDefinition` | same |
| `crates/engine/tests/core/decision_gate.rs` | `Effect` | classify the new `Effect` in the total-classification table. `crates/engine/tests/core/pb_rs1_roster_sweep.rs` has the same forward pin | **TEST** — both parse `pub enum Effect` out of the source, so they cannot be forgotten, only mis-answered. SR-36: enumerate a roster, never grep source for one |
| `crates/engine/tests/core/pb_dx28_chosen_object_roster.rs` | `Pinned against the FUNCTION` | check whether the new variant belongs — but do not expect this one to tell you | **SILENT** — it and `pb_dx26_attach_keyword_roster.rs` / `pb_dx39_source_relative_roster.rs` pin against the FUNCTION, not `pub enum Effect`. They catch a name the enum does not declare; a NEW variant slips past them |

| `crates/engine/tests/primitives/pb_dx25_counterspell_stack_shapes.rs` | `one_of_each_variant` | a representative value of the new kind, and bump the hard-coded `27` | **SILENT** — hand-maintained, with no forward pin against the enum. Its own message says it "does NOT detect a new StackObjectKind variant"; the fixture just goes quietly stale |

### The dangerous one — a closed list with no wildcard to fail on

| Where | Symbol | If you miss it |
|---|---|---|
| `crates/engine/src/rules/mana.rs` | `is_mana_producing_effect` | **SILENT, and it loses games.** It is a `matches!` over an allow-list of the ten `AddMana*` variants. A new mana-producing `Effect` that is not added simply returns `false`: the ability is not classified as a triggered mana ability (CR 605.1b), so it uses the stack and can be responded to. Nothing anywhere reddens |

**Generalise the lesson, not the row.** Every `matches!(effect, A | B | C)` and every
`find_map(|a| match a { X => .., _ => None })` over these enums is a closed list wearing an open
face. `AbilityDefinition` and `KeywordAbility` have dozens of them (narrow single-variant pickers
across `rules/` and `crates/simulator/src/legal_actions.rs`) and they are *usually* safe — a new
variant is simply not picked until you write a picker. They stop being safe the moment the list
means "all the ones that do X". Before you finish, grep your enum for `matches!` and read every
hit, asking which kind it is.

## Phase 4 — the wire (SR-8): predict the bump BEFORE you write code

`PROTOCOL_SCHEMA_FINGERPRINT` is a blake3 digest of the **transitive type closure** of the three
wire frames, and that closure reaches `Characteristics` → `Effect` → the whole card DSL. It stops
at `GameState`, which is why the two gates are separate.

- **`Effect`, `AbilityDefinition` and `KeywordAbility` are inside the closure**: adding a variant
  is a wire change and bumps `PROTOCOL_VERSION`.
- **`StackObjectKind` is not** — no `Command` or `GameEvent` variant carries one (it appears in
  those files only in doc comments). It bumps `HASH_SCHEMA_VERSION` alone. Re-check this before
  relying on it; a future event that carries a stack object would change the answer.
- **Predict which gate moves, and write the prediction in the plan before the first edit.**
  Finding out from a red gate at the end is how a batch re-pins under time pressure.
- **One pin per gate, no scattered literals** (CC-2, 2026-09-05): each version literal lives in
  exactly ONE sentinel test. A new `assert_eq!(HASH_SCHEMA_VERSION, <n>)` anywhere else is a
  review finding — assert against the constant, or not at all.
- Read the live `pub const` for the current number. Never quote one out of a doc; they drift.

| Where | Symbol | What to add | If you miss it |
|---|---|---|---|
| `crates/engine/src/rules/protocol.rs` | `PROTOCOL_SCHEMA_FINGERPRINT` | re-pin the digest and bump `PROTOCOL_VERSION` | **TEST** — `crates/engine/tests/core/protocol_schema.rs` recomputes it from source; `#[serde(skip)]` / `rename` are invisible to rustc and this is what catches them |
| `crates/engine/src/state/hash.rs` | `HASH_SCHEMA_VERSION` | bump it, append the history row | **TEST** — `crates/engine/tests/core/hash_schema.rs`; the history is append-only |

## Phase 5 — tests (SR-9a/b/c)

| Where | Symbol | What to add | If you miss it |
|---|---|---|---|
| `crates/engine/tests/core/main.rs` | `mod ` | the `mod` line for your new test file — in **your group's own** `main.rs`, one of the nine (`core`, `rules`, `combat`, `casting`, `primitives`, `scripts`, `mechanics_{a_d,e_l,m_z}`); `core` is only the example here | **SILENT** — a file in a group dir with no `mod` line is never compiled and its tests cease to exist. Demonstrated: `--test combat` reported `ok. 69 passed` with six tests missing |
| `crates/engine/tests/no_stray_test_binaries.rs` | `NON_GROUP_DIRS` | nothing — just never add a top-level `tests/*.rs` (SR-9a) | **TEST** — the gate fails the suite |
| `test-data/generated-scripts/` | — | a golden script driving the variant end-to-end, `review_status: approved`, with real `assert_state` entries | **SILENT** — the corpus gate is a **partition** check (`approved + retired == discovered`), not a coverage check. A script you never wrote is invisible to it |
| `crates/engine/tests/scripts/script_replay.rs` | `check_assertions` | implement any NEW assertion path your script uses | **TEST** — an unimplemented path is now a hard mismatch. It used to return "no mismatch" and 244 assertions went unchecked |

**The revert test.** The acceptance table's engine row asks for a revert-proven probe per fix:
write the test, revert the change, watch it go red, restore. A test that stays green under the
revert has proven nothing about the variant you added. A source-text gate that stands in for the
behaviour ships with that probe (pair-or-demote, `memory/conventions.md`).

## Phase 6 — gates

The CI bar, not the short form:

```bash
~/.cargo/bin/cargo test --workspace --no-fail-fast
~/.cargo/bin/cargo clippy --workspace --all-targets -- -D warnings   # bare `cargo clippy` skips test targets
~/.cargo/bin/cargo fmt --check
tools/check-defs-fmt.sh                                              # SR-35
```

`cargo fmt --all -- --check` exits 0 having checked **zero** of the files in
`crates/card-defs/src/defs/`: rustfmt walks `mod` declarations textually and `defs/mod.rs` is a
build-script `include!` it cannot see through. If your variant came with a card def,
`tools/check-defs-fmt.sh` is the only thing that looked at it.

## What this page is not

It is not a substitute for the plan. `primitive-impl-planner` still enumerates the sites for the
specific batch; this page is the **floor** that enumeration must clear (dispatch hygiene 6: a
brief's site list is a floor, not a census — three consecutive DX25-family batches each found the
filed scope short, and the census behind this page found `state/stack_registry.rs` and
`rules/mana.rs`, neither of which the brief that commissioned it named).

**If you find a registration point that is not on this page, add it here, in that batch.** A
checklist that is only ever read is a checklist that goes stale.
