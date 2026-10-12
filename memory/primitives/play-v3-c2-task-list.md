# Play surface v3 C2 (`scutemob-267`) — task list

<!-- last_updated: 2026-10-12 -->

Live list (reposted as an `esm task comment` at every milestone). Change class: row 3 + row 4 per new gate.

## Setup
- [x] S1 read worker.md / brief / v3 brief §1–§7 / C1 notes / conventions change-class table
- [x] S2 `npm ci`, `cargo build -p play-server`, probe server on 3041 answers
- [ ] S3 written contract for subagents (file ownership, DOM hooks, module APIs; git forbidden)
- [ ] S4 shared scaffolding by the worker: `CardFace` hooks, new CSS files imported, `ui.ctx` shape

## Interactions (one item each)
- [ ] I1 arrow targeting — `Arrows.svelte` SVG overlay at the table root
- [ ] I2 arrow targeting — drag from a decision-panel action item to a legal target card (incl. opponent cells / player seats)
- [ ] I3 arrow targeting — drag from the source card on the table; click-to-target while the target stage is open
- [ ] I4 picker driver — arrow/menu choices go through `beginExternal` + the mounted `TargetPicker`'s own select/confirm (no second protocol); `TargetPicker` stays listed
- [ ] I5 card context menu — actions the card is the source of, actions it is a legal target of, its counters; seat menu kept
- [ ] I6 blocker arrows — `BlockUi.svelte` in the decision panel (mockup `blockUi` port), drives `BlockerPicker`
- [ ] I7 rearranging — `order.js` (per-game order, append new, drop gone) + drag in `HandRow`
- [ ] I8 rearranging — drag in `YourBoard` rows (creatures / other / land stacks); hand-off to an arrow when the drag leaves the row
- [ ] I9 summoning-sick outline on creatures (`PermanentView.summoning_sick`)
- [ ] I10 auto-pass panel — `AutoPass.svelte` (off / end of turn / my next turn / step, three stop-conditions, rule sentence)
- [ ] I11 auto-pass — client stop-conditions drive `startPassUntil` (additive custom predicate in `stores.js`)
- [ ] I12 `targeted` red outline: decide from `StackItemView` (ids exposed or not) — set it or defer to C3 with the reason
- [ ] I13 integration in `TableV3` / `DecisionPanel` (mounts, hand-off wiring, float fades during a drag)

## Probes (each on a real decision, server on 3041/3042)
- [ ] P1 scenario scout: seeds/turns giving a real targeted action, a real `DeclareBlockers` with a legal block, a bot spell cast
- [ ] P2 `v3interact.py`: arrow from an action item to a target in an opponent cell; POST body equals the picker's
- [ ] P3 `v3interact.py`: arrow from the source card; click-to-target; `TargetPicker` still lists the targets
- [ ] P4 `v3interact.py`: card right-click menu (source actions, target-of actions, counters); seat menu still opens
- [ ] P5 `v3interact.py`: blocker arrows on a real `DeclareBlockers`; engine accepts the block
- [ ] P6 `v3interact.py`: rearrange hand + battlefield; survives reload; new card appends; a card that leaves drops out
- [ ] P7 `v3interact.py`: summoning-sick outline on a creature cast this turn, gone on the next own turn
- [ ] P8 `v3interact.py`: auto-pass panel — each mode stops where it says; a stop-condition fires on a real bot action
- [ ] P9 `v3sweep.py` extended (new layers never scroll / never throw); seeds 0 and 3 × 2560×1340 and 1707×880 to turn 10
- [ ] P10 C1's 35 `v3interact.py` checks still ok

## Gates
- [ ] G1 new frontend source gate(s) in `tools/play-server/src/main.rs` + one executed defeat each, recorded in the test doc; pair-or-demote
- [ ] G2 `npm run build` (frontend)
- [ ] G3 `cargo test --workspace --no-fail-fast` to a file; count delta by test NAME vs 5,355
- [ ] G4 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] G5 `cargo fmt --check` + `tools/check-defs-fmt.sh`
- [ ] G6 PROTOCOL 44 / HASH 85 unmoved (no engine / card-types / card-defs / wire file in the diff)

## Close
- [ ] C1 `esm task satisfy` 8609–8614, each the moment it is verifiably met
- [ ] C2 `/review`; HIGH + MEDIUM fixed in-cycle, LOW logged
- [ ] C3 notes `memory/play-v3-c2-2026-10-12.md` (measured table, per-interaction probe results, gate defeats, deferred for C3)
- [ ] C4 `CHANGELOG.md` entry ≤10 lines; `tools/play-server/README.md` if routes/controls text needs it
- [ ] C5 scratch deleted, probe servers stopped; commit by explicit path; `signal-ready`; session end
