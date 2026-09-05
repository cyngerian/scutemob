# LL-1 (`scutemob-255`) — task list

Batch: LL-1 — required `completeness:` marker gate (SR-39) + caster-recipient token defs.
Change class: rows 2 ("Card defs only") + 4 ("New source gate added") of the
`memory/conventions.md` change-class acceptance table. Row 4 pairs with "### Pair-or-demote".

Legend: `[ ]` todo · `[~]` in flight · `[x]` done · `[!]` blocked

## A. Recon / feasibility
- [x] A1 Read `.esm/worker.md`, `.esm/brief.md`, change-class table, landscape rules
- [x] A2 Confirm marker tally at HEAD (176 Complete / 415 partial / 147 inert / 101 known_wrong / 964 defaulted; 1,803 defs)
- [x] A3 Find the "extra" file in `ls defs/*.rs` (1,804 vs 1,803)
- [x] A4 Confirm `TokenSpec.recipient` exists (`card_definition.rs:4537`) and `PlayerTarget::ControllerOf` (:2746)
- [ ] A5 PROBE: does `ControllerOf(DeclaredTarget 0)` resolve AFTER the permanent is destroyed?
      (`resolve_player_target_list` reads `state.objects`; `move_object_to_zone` removes the id — CR 400.7)
      If it does not, STOP and post a task comment (engine change is out of class).

## B. SR-39 gate + probe (criterion 7559)
- [ ] B1 Write `crates/engine/tests/core/card_defs_completeness_marker.rs` (SR-9a: under `core/`, never top-level)
- [ ] B2 Gate rule = `authoring-report.py` `MARKER_RE` (`\bcompleteness:\s*Completeness::(\w+)`), whitespace-blind,
      recognized set {Complete, inert, partial, known_wrong}; exclude `mod.rs`
- [ ] B3 Non-vacuity floor (count checked == defs on disk)
- [ ] B4 Executed defeat: plant a marker-less def in a scratch corpus, assert the gate reddens; record in the test doc
- [ ] B5 Executed defeat #2 (the misdirection.rs trap): a def with `completeness:` only in PROSE must still fail
- [ ] B6 Add SR-39 section to `docs/engine-invariants.md`; add row to CLAUDE.md table (shave a line — 249/250)
- [ ] B7 Note the SR-38 → SR-39 numbering correction in the notes file + invariants section

## C. Sweep the 964 defaulted defs (criterion 7560)
- [ ] C1 Write the sweep script (committed under `tools/` or quoted in the notes file)
- [ ] C2 Run it: insert `completeness: Completeness::Complete,` on the 964 defaulted defs
- [ ] C3 `tools/check-defs-fmt.sh` (SR-35) + `cargo fmt --check` pass
- [ ] C4 `python3 tools/authoring-report.py` — headline unchanged except by section D
- [ ] C5 Stage `crates/card-defs/` by directory path (never `git add -A`)

## D. The six defs (criterion 7561)
- [ ] D1 `mtg-rules` MCP `lookup_card` for all six (oracle text verified, not from memory)
- [ ] D2 `beast_within.rs` — recipient + marker
- [ ] D3 `generous_gift.rs` — recipient + marker
- [ ] D4 `stroke_of_midnight.rs` — recipient, delete stale TODO + stale Partial note, re-derive marker
- [ ] D5 `emergency_eject.rs` — recipient; decide Lander expressibility from remaining clauses; re-derive marker
- [ ] D6 `saw_in_half.rs` — delete stale note; stays Partial for the halved-copy clause (name the clause)
- [ ] D7 `pongify.rs` — recipient; re-derive marker off KnownWrong
- [ ] D8 `card-batch-reviewer` agent reviews all six against oracle text
- [ ] D9 Apply review findings (`card-fix-applicator` or inline if trivial)

## E. Pins (criterion 7562)
- [ ] E1 Flip `tokens/002_beast_within_creates_beast.json` `review_status` → `approved`, delete `retirement_reason`
- [ ] E2 `SCRIPT_FILTER=002_beast_within` run passes
- [ ] E3 Direct-`Command` test, FOUR players, caster != target's controller, Beast lands with the controller (CR 701.6a)
- [ ] E4 Scripts partition gate (`crates/engine/tests/scripts/run_all_scripts.rs`) green; approved count 208 → 209

## F. Acceptance ritual (rows 2 + 4)
- [ ] F1 `cargo test --workspace --no-fail-fast` to a FILE; itemise the delta by test NAME (baseline 5,333 / 0 / 6, 73 targets)
- [ ] F2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] F3 `cargo fmt --check` + `tools/check-defs-fmt.sh`
- [ ] F4 `/review`; HIGH+MEDIUM fixed in-cycle, LOW logged to the notes file

## G. Records (criterion 7563)
- [ ] G1 Final tally (Complete/Partial/Inert/KnownWrong, zero defaulted) in `memory/primitives/ll-1-execution-notes.md`
- [ ] G2 ≤10-line `CHANGELOG.md` entry carrying the tally
- [ ] G3 `docs/mtg-engine-landscape-assessment.md` §2 fix table → point at `scutemob-255` (nothing else in that doc)
- [ ] G4 `esm task satisfy` each criterion as it is met (never batched at the end)
- [ ] G5 Delete scratchpad bench dirs; Completion Sequence
