# Review: PB-XSE1 — OOS-XS-E-1 Dies-side `exclude_self` Audit

**Date**: 2026-05-15
**Reviewer**: card-batch-reviewer (Opus, invoked via Agent tool)
**Plan reviewed**: `memory/primitives/pb-plan-XSE1.md`
**Cards reviewed**: Boggart Shenanigans, Athreos God of Passage, Meren of Clan Nel Toth
**Card-def files modified**: NONE
**Engine files modified**: NONE
**CR**: CR 109.1 (object identity — "another"), CR 603.10a (death triggers / LKI), CR 603.2 (triggered abilities)

## Verdict: AUDIT VERIFIED — plan is correct, no fixes needed

The audit plan at `memory/primitives/pb-plan-XSE1.md` is correct on every point. MCP `lookup_card` confirms all three cards' oracle text contains "another":

- **Boggart Shenanigans**: "Whenever **another** Goblin you control is put into a graveyard from the battlefield..."
- **Athreos, God of Passage**: "Whenever **another** creature you own dies, return it to your hand unless target opponent pays 3 life."
- **Meren of Clan Nel Toth**: "Whenever **another** creature you control dies, you get an experience counter."

All three card-def files confirmed to have **no** `TriggerCondition::WheneverCreatureDies` currently wired — they are intentional DSL-gap TODOs:

- Boggart Shenanigans: subtype-filter blocker (Goblin + controller You + exclude_self).
- Athreos: no opponent-pays-life alternative cost mechanism in the DSL.
- Meren: experience counters + targeted end-step graveyard return blocker.

Classification table is correct: all three should be `exclude_self: true` when eventually implemented.

## Findings

| # | Severity | File | Description |
|---|---|---|---|
| L1 | LOW | (record-keeping, not file) | AC 3893 oracle wording for Athreos is factually wrong. AC 3893 claims oracle is "each time a creature you own dies" with no "another" → false. Actual Scryfall canonical oracle (2020 update, verified via MCP) reads "Whenever another creature you own dies" → `exclude_self: true`. The plan correctly flags this. Recommend updating the acceptance-criterion record post-completion to reflect actual oracle and the resulting `exclude_self: true` classification for the future implementer. |

- **0 HIGH** findings
- **0 MEDIUM** findings
- **1 LOW** finding (record-keeping only, not card-def or engine code)

## Build / Test / Lint Baseline (independent confirmation)

- `cargo build --workspace`: clean
- `cargo test --workspace`: 2811 passed, 0 failed (matches AC 3895 baseline; no new tests added because AC 3894 is conditional on a card needing a fix and none did)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `cargo fmt --check`: clean
- `HASH_SCHEMA_VERSION = 22` (matches AC 3895)

## Recommendation

Worker may proceed to signal-ready. The single LOW finding is a record-keeping note for the coordinator; it does not require code or plan changes prior to ship.
