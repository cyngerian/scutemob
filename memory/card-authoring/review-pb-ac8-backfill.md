# Card Review: PB-AC8 backfill (6 cards)

**Reviewed**: 2026-07-09
**Cards**: 6
**Findings**: 1 HIGH, 1 MEDIUM, 1 LOW
**Scope**: read-only against `crates/engine/src/cards/defs/`; oracle text via `lookup_card` (authoritative).

---

## Card 1: Nezahal, Primal Tide (`nezahal_primal_tide.rs`)
- **Oracle match**: NEAR (one self-reference nit — see F1)
- **Types match**: YES — Legendary Creature — Elder Dinosaur
- **Mana cost match**: YES — {5}{U}{U} → `generic: 5, blue: 2`
- **P/T**: YES — 7/7
- **DSL correctness**: YES
- **TODO deletion JUSTIFIED**: YES. All four oracle clauses are expressed:
  1. "This spell can't be countered." → `cant_be_countered: true` (line 22). **This is the
     easy-to-miss clause and it IS present.**
  2. "You have no maximum hand size." → `KeywordAbility::NoMaxHandSize` (line 25).
  3. "Whenever an opponent casts a noncreature spell, draw a card." → `Triggered` with
     `WheneverOpponentCastsSpell { noncreature_only: true }`, draw 1 to `Controller` (=you). Correct.
  4. "Discard three cards: Exile Nezahal. Return it ... at the beginning of the next end step." →
     `Activated` with `Cost::Sequence([DiscardCard;3])` + `ExileWithDelayedReturn { target: Source,
     return_timing: AtNextEndStep, return_tapped: true, return_to: Battlefield }`. Correct
     (returns under owner's control by CR default; tapped flag set).
- **Findings**:
  - F1 (LOW, KI-18): `oracle_text` field (line 19) reads "Exile **Nezahal, Primal Tide**." but the
    authoritative MCP oracle reads "Exile **Nezahal**." (short self-reference). The file's own header
    comment (line 6) uses the short form, so the field is internally inconsistent too. Cosmetic; does
    not affect behavior. (If the intent is Scryfall's full-name self-reference convention, confirm
    against Scryfall directly — MCP returned the short form.)

## Card 2: Toski, Bearer of Secrets (`toski_bearer_of_secrets.rs`)
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Squirrel
- **Mana cost match**: YES — {3}{G} → `generic: 3, green: 1`
- **P/T**: YES — 1/1
- **DSL correctness**: YES
- **TODO deletion JUSTIFIED**: YES. All four clauses expressed:
  1. "This spell can't be countered." → `cant_be_countered: true` (line 17). Present — not missed.
  2. "Indestructible" → `KeywordAbility::Indestructible`.
  3. "Toski attacks each combat if able." → `KeywordAbility::MustAttackEachCombat`.
  4. "Whenever a creature you control deals combat damage to a player, draw a card." →
     `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: None }`, draw 1 to `Controller`.
     `filter: None` is correct here (no subtype/token restriction — any creature you control).
- **Findings**: none. Clean.

## Card 3: Curiosity Crafter (`curiosity_crafter.rs`)
- **Oracle match**: YES
- **Types match**: YES — Creature — Bird Wizard (no supertype; correct)
- **Mana cost match**: YES — {3}{U} → `generic: 3, blue: 1`
- **P/T**: YES — 3/3
- **In-scope half correct**: YES — `Flying` + `NoMaxHandSize`.
- **Marker still present**: YES (lines 7-9, 24) — card is correctly *not* claimed clean.
- **Findings**:
  - **F1 (HIGH, KI-3 — stale ENGINE-BLOCKED marker)**: The retained TODO claims "No `TriggerCondition`
    variant restricts the combat-damage-dealer to token creatures specifically." This is very likely
    **stale**. The trigger it needs is
    `WheneverCreatureYouControlDealsCombatDamageToPlayer { filter: Some(...) }` — the same variant used
    (with a subtype filter) by `rakish_heir.rs:20-25`, `ingenious_infiltrator.rs`, `yuriko...`,
    `stensia_masquerade.rs`. The `filter` is a `TargetFilter`, and **two other def files document that
    `TargetFilter`'s token field IS checked on the combat-damage path**:
      - `metastatic_evangel.rs:18-23`: "is_token in TargetFilter is only checked in
        combat_damage_filter paths; for ETB trigger matching it is silently ignored."
      - `baron_bertram_graywater.rs:26-27`: "`TargetFilter.is_token` ... is checked only in the
        `combat_damage_filter` path (card_definition.rs)".
    Curiosity Crafter's trigger is **exactly a combat-damage-filter path**, so a token-only restriction
    should be expressible today, e.g.
    `filter: Some(TargetFilter { is_token: Some(true), ..Default::default() })` (field name/type to be
    confirmed by the engine-source reviewer). If confirmed wired, this card should be **FULLY authored**,
    not PARTIAL — leaving it PARTIAL understates coverage and leaves an expressible ability unimplemented,
    violating invariant #9 (ability silently never fires). Recommend the engine-source agent verify the
    `is_token` field name and combat-damage wiring; if present, implement the filter and delete the marker.

## Card 4: Niv-Mizzet, Visionary (`niv_mizzet_visionary.rs`)
- **Oracle match**: YES
- **Types match**: YES — Legendary Creature — Dragon Wizard
- **Mana cost match**: YES — {4}{U}{R} → `generic: 4, blue: 1, red: 1`
- **P/T**: YES — 5/5
- **In-scope half correct**: YES — `Flying` + `NoMaxHandSize`.
- **Marker still present**: YES (lines 10-13, 32-33).
- **Gap accuracy**: Marker is accurate. "Whenever a source you control deals noncombat damage to an
  opponent, you draw that many cards" requires (a) an any-source noncombat-damage `TriggerCondition`
  and (b) an `EffectAmount` equal to the damage dealt by the triggering event. Neither appears
  expressible in the current DSL; distinct from the fixed-amount draw triggers used elsewhere. Genuine
  gap — correctly left PARTIAL.
- **Findings**: none beyond the retained (accurate) marker.

## Card 5: Hellkite Tyrant (`hellkite_tyrant.rs`)
- **Oracle match**: YES
- **Types match**: YES — Creature — Dragon (NOT legendary; correct — no supertype)
- **Mana cost match**: YES — {4}{R}{R} → `generic: 4, red: 2`
- **P/T**: YES — 6/5
- **`Effect::WinGame` gating**: CORRECT. Gated via `intervening_if: Some(Condition::
  YouControlNOrMoreWithFilter { count: 20, filter: has_card_type Artifact })` (lines 29-35), NOT via a
  nonexistent `WinGame.condition` field. Matches oracle "if you control twenty or more artifacts, you
  win the game." CR 603.4 re-check applies. This win-con is **reachable and correct** (does not depend
  on the blocked ability).
- **Gap accuracy**: The retained marker (lines 19-21) for "gain control of all artifacts that player
  controls" on combat damage is accurate — no mass gain-control-of-type-targeting-damaged-player effect
  exists. Genuine gap; correctly left PARTIAL.
- **Findings**: none. Solid PARTIAL (implemented half is fully correct).

## Card 6: Simic Ascendancy (`simic_ascendancy.rs`)
- **Oracle match**: YES
- **Types match**: YES — Enchantment (no supertype; no P/T — correct)
- **Mana cost match**: YES — {G}{U} → `green: 1, blue: 1`
- **`Effect::WinGame` gating**: CORRECT. Gated via `intervening_if: Some(Condition::SourceHasCounters
  { counter: CounterType::Custom("growth"), min: 20 })` (lines 52-55), NOT a nonexistent condition
  field. Matches oracle "if this enchantment has twenty or more growth counters on it, you win the game."
- **`CounterType::Custom("growth")` convention**: CORRECT/consistent. Matches the established custom-counter
  convention: `dragons_hoard.rs` ("gold"), `the_one_ring.rs` ("burden"), `strixhaven_stadium.rs` ("point"),
  `ominous_seas.rs` ("foreshadow"). No dedicated `growth` enum variant needed.
- **Activated ability**: CORRECT. `{1}{G}{U}: Put a +1/+1 counter on target creature you control` →
  `Cost::Mana({1}{G}{U})`, target `TargetCreatureWithFilter { controller: You }`, `AddCounter
  { PlusOnePlusOne, count: 1 }` on declared target. Functional and correct.
- **Marker still present**: YES (lines 33-47).
- **Gap accuracy**: Accurate. The "put that many growth counters" effect needs an `EffectAmount` reading
  the triggering counter count (Master-Biomancer-style multi-placement); no such variant exists.
- **Findings**:
  - **F1 (MEDIUM)**: **The win condition is currently UNREACHABLE in normal play.** Because the
    counter-placement trigger (the only thing that adds growth counters) is blocked, growth counters can
    never reach 20 except via a manual test-harness setup, so the `WinGame` ability is effectively inert
    (dead) until that gap closes. This is **not wrong game state** — it fails safe: with the intervening-if
    false at trigger time (CR 603.4), the ability never even goes on the stack, so no spurious wins or
    stack objects are produced. The card is honestly marked PARTIAL, so it will not be counted clean and
    does not corrupt the authoring-report coverage number or violate invariant #9 (it won't reach a real
    game deck while PARTIAL). Acceptable to ship as authored, but flag for tracking: functionally the card
    currently provides only its activated +1/+1 ability; the win-con and its feeder trigger should ship
    together. No change required now.

---

## Summary
- **Cards with issues**: Curiosity Crafter (HIGH — stale marker), Simic Ascendancy (MEDIUM — inert
  win-con), Nezahal (LOW — oracle self-reference nit).
- **Clean / solid**: Toski (clean, TODO deletion justified), Hellkite Tyrant (solid PARTIAL, implemented
  half fully correct), Niv-Mizzet Visionary (accurate PARTIAL).
- **TODO deletions verified justified**: Nezahal (all 4 clauses incl. can't-be-countered) and Toski
  (all 4 clauses incl. can't-be-countered) are correctly full — no residual clause was missed. Deleting
  their markers is justified.
- **Effect::WinGame**: both Hellkite and Simic gate via `intervening_if` (correct); neither invents a
  `WinGame.condition` field.
- **Action recommended**: Engine-source reviewer to confirm whether `TargetFilter`'s token field is
  wired into the combat-damage-filter path (per in-repo docs it is). If so, Curiosity Crafter's
  ENGINE-BLOCKED marker is stale and the card can be fully authored.
