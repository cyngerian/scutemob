# Primitive Batch Review: PB-EF3b — Granted keyword-triggers fire (Melee / Battle Cry / Annihilator)

**Date**: 2026-07-18
**Reviewer**: primitive-impl-reviewer (Opus)
**Commit reviewed**: `43e73b32`
**CR Rules**: 702.121 (Melee), 702.91 (Battle Cry), 702.86 (Annihilator), 613.1f (Layer 6 grants), 508.5 (defending player), 603.2/603.3, 113.7a (LKI)
**Engine files reviewed**: `state/builder.rs` (helper + routing), `rules/layers.rs` (reconciliation), `rules/abilities.rs` (3 tag reads)
**Card defs reviewed**: `adriana_captain_of_the_guard.rs` (NEW, Complete), `skyhunter_strike_force.rs` (NEW, partial) — 2 total
**Tests reviewed**: `tests/primitives/pb_ef3b_granted_keyword_triggers.rs` (8 tests)

## Verdict: needs-fix (LOW-only)

The implementation is correct and matches CR text on every load-bearing path. The helper
extraction is sound (both the printed path and the reconciliation path now call the *same*
`derived_attack_trigger_for_keyword`, so description-dedup structurally cannot drift), the
reconciliation is inserted at the correct post-layer/post-merge boundary and sees the final
keyword set, the three raw→resolved tag reads are index-correct and None-tolerant, both card
defs match oracle text, and all 8 tests are non-vacuous with real decoys. No schema bump was
needed and none was made (the synthesis lands only in computed `Characteristics`, which is never
serialized or hashed). No HIGH or MEDIUM findings. Three LOW notes below (documented set-model
limitation, a pre-existing `RemoveKeyword` asymmetry surfaced by this work, and minor test gaps)
— none block collection; item 2 warrants an OOS seed.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | LOW | `rules/layers.rs:442-459` | **Set-model single-fire vs CR 702.x.b.** Printed+granted of the *same* keyword collapse to one OrdSet entry → one derived trigger. Honestly documented + test-pinned; architecture-consistent. **Fix:** none required; keep as documented limitation. |
| 2 | LOW | `rules/layers.rs:1207-1209` | **`RemoveKeyword` does not strip a stale derived trigger.** `RemoveKeyword(kw)` removes only from `chars.keywords`; a printed trigger-keyword's derived def stays in base `triggered_abilities`, so a printed Melee still fires after `RemoveKeyword(Melee)`. Pre-existing (not introduced here). **Fix:** file OOS-EF3b-3 to have `RemoveKeyword` also drop the matching derived def (or have reconciliation reconcile removals, not just additions). |
| 3 | LOW | `tests/primitives/pb_ef3b_granted_keyword_triggers.rs` | **Test gaps:** no Melee-token case (`make_token` now benefits from reconciliation), no planeswalker-attack Melee case, no `RemoveKeyword`-after-grant case. **Fix:** optional follow-up tests; not required for collection. |

## Card Definition Findings

None. Both defs match oracle text; Skyhunter's partial marker is truthful.

### Finding Details

#### Finding 1: Set-model single-fire vs CR 702.121b/702.91b/702.86b

**Severity**: LOW
**File**: `rules/layers.rs:442-459`
**CR Rule**: 702.121b/702.91b/702.86b — "If a creature has multiple instances of [keyword], each triggers separately."
**Issue**: `Characteristics.keywords` is `OrdSet<KeywordAbility>`, so printed Melee + a second, independently-granted Melee collapse to one set entry and the reconciliation (dedup by exact description equality) produces exactly one derived trigger — a single +1/+1, not +2/+2. This is genuinely wrong game state in the rare case of two independent same-keyword sources on one creature (two Melee anthems; printed + granted Annihilator N with equal N). It is, however, **consistent with how the engine models all keyword redundancy** (double flying, double lifelink, etc. all collapse), so honoring 702.x.b would require re-architecting keyword storage from a set to a multiset — far outside a correctness PB. The limitation is documented in the reconciliation comment with CR citations and pinned by `test_ef3b_no_double_fire_printed_plus_granted_melee`. Adriana herself is unaffected: `OtherCreaturesYouControl` excludes her, so her printed Melee is never double-granted.
**Fix**: None required. Correct scoping. Optionally record the multiset-keyword model as a long-horizon OOS seed if a card is ever printed that stacks same-keyword trigger sources intentionally.

#### Finding 2: `RemoveKeyword` leaves a stale printed derived trigger

**Severity**: LOW
**File**: `rules/layers.rs:1207-1209`
**CR Rule**: 613.1f (Layer 6 ability removal)
**Issue**: `LayerModification::RemoveKeyword(kw)` executes only `chars.keywords.remove(kw)`. For a **printed** trigger-keyword the derived `TriggeredAbilityDef` lives in base `chars.triggered_abilities` (built by `builder.rs`), and `RemoveKeyword` never touches that vec — so `collect_triggers_for_event`, reading resolved chars, still finds and fires the trigger after the keyword was supposedly removed. This is **pre-existing** (true for every printed trigger-keyword before this PB) and not made worse by the change; the new reconciliation correctly appends nothing for a removed keyword. `RemoveAllAbilities` does clear `triggered_abilities` (L1204), which is why the Humility test passes — the asymmetry is only in the single-keyword `RemoveKeyword` path. This PB is the natural place to note it since it formalizes the keyword→derived-trigger relationship.
**Fix**: File OOS-EF3b-3 — either (a) have `RemoveKeyword(kw)` also drop any `triggered_abilities` entry whose description matches `derived_attack_trigger_for_keyword(kw)`, or (b) drive the reconciliation from keyword presence for *both* directions (rebuild derived triggers from the final keyword set rather than append-only). Out of scope for PB-EF3b.

#### Finding 3: Minor test gaps

**Severity**: LOW
**File**: `tests/primitives/pb_ef3b_granted_keyword_triggers.rs`
**Issue**: Reconciliation now also fixes Melee/BattleCry/Annihilator **tokens** (`make_token` never synthesized derived triggers) — an unasserted bonus. No planeswalker-attack Melee test (ruling: Melee triggers when attacking a planeswalker but counts only opponents attacked with a creature). No `RemoveKeyword`-after-grant test (relevant to Finding 2). The 8 present tests cover the primary matrix well; these are additive.
**Fix**: Optional follow-up tests. Not required for collection.

## Detailed Verification

**1. Helper extraction (byte-identical).** `derived_attack_trigger_for_keyword` (`builder.rs:1106-1177`) returns the Annihilator/BattleCry/Melee defs; `builder.rs:481-483` routes the printed loop through it, and `layers.rs:450` calls the identical function. Because both the printed and reconciliation paths now share one source of truth, description-dedup **cannot** drift regardless of the original text — that class of bug is designed out. Behavioral parity for printed keywords is guarded by the pre-existing `mechanics_m_z/melee.rs` (32 P/T assertions), `mechanics_a_d/battle_cry.rs`, `mechanics_a_d/annihilator.rs` suites plus the new `test_ef3b_printed_melee_unchanged_regression`. Descriptions match CR text verbatim; Melee's `effect: None` + `"Melee (CR 702.121a):…"` prefix satisfies the `effect.is_none() && description.starts_with("Melee")` dispatch tag.

**2. Reconciliation soundness.** Inserted at `layers.rs:436-459`, after the full layer loop *and* the merge-component integration (L413-435), immediately before `Some(chars)` — it sees the final keyword set. (a) printed-only: base has the def → `already` → skip; (b) granted-only: base lacks it → append → fires; (c) printed+granted: one set entry, base has def → skip (no double); (d) Humility/`RemoveAllAbilities` empties `keywords` (L1201) *and* `triggered_abilities` (L1204) → loop iterates nothing → appends nothing. Merged permanents: the component's own builder-built Melee def (copied at L426) has the identical description → dedup skips. Idempotent: `calculate_characteristics` clones from `obj.characteristics` each call and never writes back, so appends never accumulate. Borrow handled by collecting keywords into a `Vec` first (L448). Cannot append the wrong def — Annihilator(2)≠Annihilator(3) by description, so distinct-N instances both fire correctly.

**3. Tag-read fix.** All three sites (Myriad `abilities.rs:3595`, Provoke `:3621`, Melee `:3670`) switched from `state.fizzle_object(t.source)` (raw base) to `calculate_characteristics(state, t.source)` (resolved), indexing `t.ability_index` which is set against resolved chars at collection time — index-correct. `calculate_characteristics` returns `Option` (not `expect_`), preserving None-tolerance per CR 113.7a. `t.source` is the attacker (`collect_triggers_for_event(SelfAttacks, Some(*attacker_id))`). No behavior change for printed keywords (raw==resolved at those indices). The collect call and the tag call recompute the same deterministic OrdSet ordering, so the granted Melee's index is stable between them.

**4. CR fidelity.** Helper defs match 702.121a/702.91a/702.86a verbatim. The set-model single-fire deviation from 702.x.b is documented and scoped (Finding 1). Annihilator defending-player is stamped independently at `abilities.rs:3580-3582` from the `AttackTarget` (CR 508.5), unaffected by ability_index — confirmed correct for granted Annihilator by `test_ef3b_granted_annihilator_via_anthem`.

**5. Card defs vs oracle.** Adriana (MCP verified): {3}{R}{W} Legendary Creature — Human Knight 4/4, "Melee" + "Other creatures you control have melee." Def: printed `Keyword(Melee)` + `Static { AddKeyword(Melee), OtherCreaturesYouControl, WhileSourceOnBattlefield }` — `OtherCreaturesYouControl` correctly excludes Adriana from a second Melee (pinned: `test_ef3b_adriana…` asserts she is 5/5 not 6/6). Complete is correct. Skyhunter (MCP verified): {2}{W} Cat Knight 2/2, Flying + Melee + Lieutenant anthem. Def models Flying + Melee, **omits** the Lieutenant clause (no `Condition`/`TargetFilter` expresses "you control your commander"), marked `partial` with a truthful blocker note citing OOS-EF3b-1. Omission (not a wrong unconditional anthem) is the correct W6 call. Registry gate satisfied — Melee registers a derived trigger, so the def is not inert.

**6. Test rigor.** All 8 tests cite CR and assert post-resolution P/T + trigger/stack counts. Decoys are real: Test 1 (2/2 without reconciliation), Test 5 (4/4 if `already` dedup removed), Test 6 (double-append guard), Test 8 (Humility path). Isolation via `SingleObject` grants in Tests 2/3 is correct. Test 7 exercises the full registry→enrich→static-registration→attack chain.

**7. Schema.** No new type/variant/field, no serde change. Synthesis lands only in computed `Characteristics` (never serialized, never hashed — stops short of `GameState`). `PROTOCOL_VERSION`/`PROTOCOL_SCHEMA_FINGERPRINT`/`HASH_SCHEMA_VERSION` correctly untouched (no occurrence in `layers.rs`). Correct — no bump needed.

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| 702.121a (Melee fires + pump) | Yes | Yes | test 1, 2, 6, 7 |
| 702.121b (each instance separately) | No (set model) | Yes (single-fire pinned) | Finding 1 — documented limitation |
| 702.91a (Battle Cry +1/+0 others) | Yes | Yes | test 3 |
| 702.86a (Annihilator sac) | Yes | Yes | test 4 |
| 508.5 (defending player) | Yes | Yes | test 4 (granted Annihilator) |
| 613.1f (Layer 6 grant) | Yes | Yes | tests 1-7 (grant), 8 (removal) |
| RemoveAllAbilities strips trigger | Yes | Yes | test 8 |
| RemoveKeyword strips trigger | No (pre-existing) | No | Finding 2 → OOS-EF3b-3 |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| adriana_captain_of_the_guard | Yes | 0 | Yes | Complete; anthem excludes self, verified |
| skyhunter_strike_force | Yes (Flying+Melee) | 0 | Yes (omission) | partial; Lieutenant omitted, truthful marker, OOS-EF3b-1 |

## OOS / Follow-up

- OOS-EF3b-1 (Lieutenant "control your commander" grant condition) — filed in `ef-batch-plan-2026-07-17.md` §7. Confirmed present.
- OOS-EF3b-2 (extend helper to full synthesized keyword-trigger set for granted Myriad/Provoke/etc.) — filed. Confirmed present.
- **OOS-EF3b-3 (NEW, recommended)** — `RemoveKeyword(kw)` should also drop the matching derived `TriggeredAbilityDef` (Finding 2). Not filed yet.
