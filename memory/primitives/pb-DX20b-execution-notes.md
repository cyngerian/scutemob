# PB-DX20b — execution notes

> `scutemob-222`, v4 queue rank 9. Closes **OOS-DX20-10** (self-labelled HIGH) and **OOS-DX20-5**.
> Registry of record: `docs/audits/decision-point-audit.md`.

---

## §0 — Stage 0, written BEFORE any production line changed

Everything in this section was committed before the first engine edit. Nothing below is
back-filled; the corrections that execution forced are recorded in later sections and the
first drafts are left standing so the deltas are readable.

### §0.1 Pre-edit baseline (measured, not remembered)

`cargo test --workspace --no-fail-fast` to a file on this branch, before any edit:

- **4,991 passed / 0 failed / 5 ignored**, **59** result-producing targets.
- This **reproduces PB-DX50's close pin exactly** (4,991 / 0 / 5, 59 targets), so the branch
  starts where the last collect said it did.
- Passing test NAME set captured to a file for the close-out set-diff (4,974 names; the gap to
  4,991 is duplicate names across targets, which a set collapses — stated so the delta arithmetic
  is not read as a discrepancy).

### §0.2 Wire prediction (predicted here, gate-computed later)

| axis | at HEAD | predicted | confidence | reason |
|---|---|---|---|---|
| `PROTOCOL_VERSION` | **40** | **41** — ONE bump | HIGH | `KeywordAbility` is a declared protocol closure root (`protocol_schema.rs:103`); `KeywordAbility::Enchant(EnchantTarget)` → `EnchantTarget::Filtered(EnchantFilter)`, so adding a field to `EnchantFilter` changes a declaration inside the closure. The fingerprint is a **shape** digest, so a new field moves it. |
| `PROTOCOL` closure type count | **98** | **98 — unchanged** | HIGH | the new field is `Vec<CardType>`; `CardType` is already in the closure via the existing `has_card_type: Option<CardType>`. No new type is reachable. |
| `HASH_SCHEMA_VERSION` | **79** | **80** — ONE bump | HIGH | `impl HashInto for EnchantFilter` (`hash.rs:1714`) gains a line, and `EnchantTarget` is hashed at `:1696` inside `Characteristics.keywords`. The declaration text moves, so the hash-schema fingerprint moves. |
| `HASH` closure type count | **131** | **131 — unchanged** | HIGH | same reason as the protocol count. |

**ONE bump each for the whole PB.** The stop-condition, stated in advance: if either gate moves
in a way the field addition does not explain, or if either does **not** move, stop and re-derive
rather than re-pinning.

Card-def edits move neither: a def is data, not a declaration, and `CardRegistry`/`CardDefinition`
are `CLOSURE_MUST_NOT_CONTAIN` members on the protocol side.

### §0.3 Site census — the memo says three; re-derived here because a site list is a floor

The v4 memo row 9 cell says *"three sites"*. Re-derived at HEAD by reading every consumer of
`EnchantTarget`/`EnchantFilter` outside the card-def corpus and the test tree:

| # | site | what it does | is it an independent arithmetic? |
|---|---|---|---|
| 1 | `casting::enchant_target_to_requirement` (`casting.rs:5640`) | lowers `EnchantTarget` → `TargetRequirement`; `Filtered(f)` maps six fields onto `TargetFilter` | **YES** — this is the cast/offer arithmetic |
| 2 | `sba::enchant_filter_matches` (`sba.rs:1052`), reached via `sba::matches_enchant_target` (`:1019`) | hand-rolled six-field predicate over `Characteristics` | **YES** — a SECOND, independent copy of the same arithmetic |
| 3 | the CR 303.4a cast gate (`casting.rs:3862-3910`) | calls `sba::matches_enchant_target` | **NO** — a consumer of site 2, not a third arithmetic |
| 4 | `sba.rs:1157-1180`, the CR 704.5m SBA | calls `sba::matches_enchant_target` | **NO** — a consumer of site 2 |
| 5 | `queries::spell_target_requirements` (`queries.rs:167`) | calls `casting::card_def_target_requirements` | **NO** — a consumer of site 1, by PB-DX20's own design |

**Correction to the memo's cell.** "Three sites" is right about the *count of places that
mention the predicate* and wrong about the shape: there are **two arithmetics and three
consumers**, and the consumers are already shared. The expressiveness gap therefore has to be
closed in **two** places or the two arithmetics drift on the new field — which is exactly the
failure mode AC 7308's *"ONE arithmetic"* clause forbids.

**Shipped design (decided at stage 0):** one `pub(crate) fn enchant_filter_to_target_filter`
does the lowering, and `sba::enchant_filter_matches` is rewritten to *call it* and then hand off
to `effects::matches_filter` — the same predicate `validate_object_satisfies_requirement` runs on
the cast path. After that there is exactly ONE place that knows what an `EnchantFilter` field
means. The controller clause stays split, because `matches_filter` takes only `Characteristics`
and controller is not a characteristic — that split is stated, not hidden, and both halves read
the same `EnchantControllerConstraint`.

`sba.rs` keeping its own *entry point* is deliberate and is a different property from the one
being unified: PB-DX20 kept the CR 303.4a gate calling the SBA's predicate so that cast-time and
SBA-time agree. That property survives — it is now *guaranteed by construction* rather than by
two hand-written copies agreeing.

### §0.4 Census predictions (to be refuted or confirmed by execution)

Predicted before the roster test was written:

- Members needing **OR over card types**: `imprisoned_in_the_moon` (`Complete`, deck-legal, LIVE)
  and `kayas_ghostform` (`partial`).
- Members needing **a controller clause only**: `breath_of_fury` (`partial`) — printed
  *"Enchant creature you control"*, declared `EnchantTarget::Creature`. **This is a THIRD census
  member and neither seed row nor the v4 memo cell names it.** It needs no new expressiveness at
  all: `EnchantFilter.controller` has existed since PB-DX20.
- **Coverage flips predicted: ZERO, on both defs, and the memo's cell is wrong about one of
  them.** Row 9 predicts *"0 flips; +1 `partial` unblocked (`kayas_ghostform`)"*.
  `kayas_ghostform`'s own `Completeness::partial` note already says the Enchant line is **"NOT
  blocked"** and names a different blocker — a trigger keyed to the *enchanted permanent's* zone
  change plus a return from graveyard-or-exile. Repairing the Enchant line does not touch that,
  so the def stays `partial`. `breath_of_fury` likewise stays `partial` (Aura re-attachment).
  `imprisoned_in_the_moon` stays `Complete` — now honestly. Predicted coverage: **1,137/1,803 =
  63.1%, unmoved, 0 flips.**
