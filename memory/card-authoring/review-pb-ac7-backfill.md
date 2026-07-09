# Card Review: PB-AC7 backfill (type-changing & ability-removal)

**Reviewed**: 2026-07-09
**Cards**: 7 (5 CLEAN, 2 PARTIAL) + integration test file
**Findings**: 0 HIGH, 1 MEDIUM, 4 LOW

All oracle text quoted below is from `mcp__mtg-rules__lookup_card` (authoritative).

---

## Card 1: Kenrith's Transformation (CLEAN)
- **Oracle match**: YES — `{1}{G}`, Enchantment — Aura, text exact.
- **Types match**: YES — Aura subtype present; `Keyword(Enchant(Creature))` present.
- **DSL correctness**: YES.
- **Findings**: none.
  - Layer composition correct: `RemoveAllAbilities` (L6), `SetCardTypes({Creature})` +
    `SetCreatureTypes({Elk})` (L4, preserves supertypes per CR 205.1a), `SetColors({Green})`
    (L5), `SetPowerToughness 3/3` (L7b). Uses `SetCardTypes`/`SetCreatureTypes`, NOT
    `SetTypeLine` — satisfies the ruling "The creature keeps any supertypes it has."
  - ETB "draw a card" trigger present (`WhenEntersBattlefield` → `DrawCards Controller 1`).
  - No TODO/ENGINE-BLOCKED markers (verified).

## Card 2: Eaten by Piranhas (CLEAN)
- **Oracle match**: YES — `{1}{U}`, Enchantment — Aura, Flash + Enchant + type-change text exact.
- **Types match**: YES.
- **DSL correctness**: YES.
- **Findings**: none.
  - `Flash` + `Enchant(Creature)` keywords present. `RemoveAllAbilities` (L6),
    `SetCardTypes({Creature})` + `SetCreatureTypes({Skeleton})` (L4), `SetColors({Black})`
    (L5), `SetPowerToughness 1/1` (L7b). Oracle here DOES change color ("black … loses all
    other colors") — correctly modeled, unlike Darksteel. Supertypes preserved (no `SetTypeLine`).
  - Ruling "loses Equipment/Vehicle/Cave subtypes too" is handled via CR 205.1a correlated-
    subtype drop on `SetCardTypes` + `SetCreatureTypes`. No TODO markers.

## Card 3: Darksteel Mutation (CLEAN)
- **Oracle match**: YES — `{1}{W}`, Enchantment — Aura, text exact.
- **Types match**: YES.
- **DSL correctness**: YES.
- **Findings**: none.
  - Correct: does NOT set colors (ruling: "doesn't affect the enchanted creature's colors").
  - `RemoveAllAbilities` is listed BEFORE `AddKeyword(Indestructible)` (both L6). Per
    `register_static_continuous_effects` timestamp-in-ability-vec-order, Remove gets the earlier
    timestamp; Indestructible is granted later and survives (CR 613.7). Verified by the
    integration test `test_darksteel_mutation_full_integration` (asserts Indestructible present,
    Flying gone).
  - `SetCardTypes({Artifact, Creature})` + `SetCreatureTypes({Insect})` (L4), base P/T 0/1 in
    L7b (a set, not a modify) — correct.

## Card 4: Sram, Senior Edificer (CLEAN)
- **Oracle match**: YES — `{1}{W}`, Legendary Creature — Dwarf Advisor, 2/2, text exact.
- **Types match**: YES — Legendary supertype present (`full_types(&[SuperType::Legendary], …)`).
- **DSL correctness**: YES.
- **Findings**: none.
  - `WheneverYouCastSpell.spell_subtype_filter = [Aura, Equipment, Vehicle]` — OR-semantics over
    exactly the three named subtypes. `noncreature_only: false` is CORRECT: the trigger must
    fire for any spell carrying those subtypes regardless of creature-ness (e.g. a Reconfigure
    artifact-creature Equipment). Integration test covers all three positive cases + a vanilla-
    creature negative case.

## Card 5: Leaf-Crowned Visionary (CLEAN)
- **Oracle match**: YES — `{G}{G}`, Creature — Elf Druid, 1/1, text exact.
- **Types match**: YES.
- **DSL correctness**: YES.
- **Findings**: none.
  - Static: `OtherCreaturesYouControlWithSubtype(Elf)` +1/+1 (L7c) — "Other" exclusion built into
    the filter. Trigger: `WheneverYouCastSpell.spell_subtype_filter = [Elf]` →
    `MayPayThenEffect { cost: {G}, then: DrawCards }` — correct may-pay structure (CR 118.12).
    Not restricted to creature spells (`noncreature_only: false`), matching "cast an Elf spell."

## Card 6: Final Showdown — mode 0 (PARTIAL)
- **Oracle match**: YES — `{W}` Instant, Spree, three modes exact.
- **DSL correctness**: YES for the authored clauses.
- **Findings**:
  - F-FS1 (LOW / informational): Mode 0 now-expressible clause is oracle-accurate —
    `ApplyContinuousEffect { layer: Ability, modification: RemoveAllAbilities,
    filter: AllCreatures, duration: UntilEndOfTurn }`. `AllCreatures` correctly means every
    creature in play (oracle "All creatures"), not just yours.
  - F-FS2 (LOW): Mode 1 ("Choose a creature you control. It gains indestructible until end of
    turn.") is a documented no-op (`Effect::Sequence(vec![])`, ENGINE-BLOCKED, OOS-AC7-2). The
    marker is ACCURATE and NARROWED. Verified genuine gap: `EffectTarget`
    (card_definition.rs:2284) has no resolution-time "choose a permanent you control" variant —
    only `DeclaredTarget`, `AllCreatures`, `AllPermanentsMatching`, etc. A real target would be
    CR-wrong (ruling: "The second mode … doesn't target the creature"); `AllCreatures you control`
    would be wrong game state with >1 creature. Residual: selecting mode 1 grants nothing — this
    is an unavoidable documented partial (Spree modes are positional and cannot be omitted), NOT
    a silent approximation. Acceptable under partial policy.
  - Spree dual-def satisfied: `Keyword(Spree)` present AND `ModeSelection { min_modes: 1,
    max_modes: 3, allow_duplicate_modes: false, mode_costs: [{1},{1},{3}{W}{W}] }`. Mode costs
    and modality match oracle. Mode 2 `DestroyAll` creatures correct.

## Card 7: Vraska, Betrayal's Sting — −2 (PARTIAL)
- **Oracle match**: YES — `{4}{B}{B/P}` (generic 4 + black 1 + phyrexian Single(Black); MV 6),
  Legendary Planeswalker — Vraska, loyalty 6, all three ability texts exact.
- **DSL correctness**: mostly; one verification gap (below).
- **Findings**:
  - F-VR1 (MEDIUM — verification gap / legal-but-wrong risk) — **RESOLVED 2026-07-09**: The −2
    grants the Treasure a `{T}, Sacrifice: add any color` mana ability while also applying
    `RemoveAllAbilities`. All five `ApplyContinuousEffect` calls run inside one `Effect::Sequence`,
    and the handler (effects/mod.rs `let ts = state.timestamp_counter;`) does NOT advance the
    timestamp — so `RemoveAllAbilities` and `AddManaAbility` share the SAME timestamp. Survival of
    the mana ability then relies on stable-sort insertion order for equal timestamps
    (layers.rs `sort_by_key(|e| e.timestamp)` in `toposort_with_timestamp_fallback`, stable;
    Remove pushed before Add).
    - **Verification**: Confirmed CORRECT, not merely plausible. Traced the full chain:
      `active_effects` (layers.rs `calculate_characteristics`) iterates `state.continuous_effects`
      (a `VecDeque`, `push_back`-appended) preserving push order; the per-layer `layer_effects`
      filter preserves that order; `toposort_with_timestamp_fallback` has no `depends_on` edge
      between `RemoveAllAbilities` and `AddManaAbility` (independent effects), so it falls back to
      a stable sort on the tied timestamp, which preserves push order — Remove applies before Add.
      `RemoveAllAbilities` (layers.rs) clears `chars.mana_abilities` to empty; `AddManaAbility`
      (applied after) pushes the granted ability back. Net result: exactly one mana ability
      survives. Cross-checked against MCP: CR 613.7's timestamp/APNAP tie-break rules govern
      ordering between *separate* resolutions/objects, not clauses within one card's own text —
      Vraska's oracle "loses all OTHER card types and abilities" and the analogous, already-tested
      Darksteel Mutation ruling ("loses all abilities **except** indestructible") both confirm the
      granted ability is meant to survive; the engine's Remove-then-Add push-order pattern
      (already established and tested via Darksteel Mutation, `AddKeyword` instead of
      `AddManaAbility`) achieves that correctly here too, just via equal (not distinct) timestamps.
    - **Fix applied**: Added `test_vraska_betrayals_sting_minus2_full_integration` to
      `crates/engine/tests/pb_ac7_card_integration.rs` — resolves the real -2 ability via
      `Command::ActivateLoyaltyAbility` and asserts the target is `Artifact` / `Treasure`,
      retains its supertypes (Legendary), lost all keywords/activated/triggered abilities, and has
      exactly one surviving mana ability with the correct shape
      (`requires_tap`/`sacrifice_self`/`any_color`, empty `produces`).
    - **Non-vacuousness proven**: temporarily reversed the push order of the `RemoveAllAbilities`
      and `AddManaAbility` `ApplyContinuousEffect`s in `vraska_betrayals_sting.rs` — the test
      failed as expected (`mana_abilities.len()`: expected 1, got 0), confirming the granted
      ability is stripped when push order is wrong. Reverted immediately after confirming the
      failure; original order restored and test re-verified passing.
    - **Code comments added**: documented the equal-timestamp/stable-sort reliance at both sites —
      `effects/mod.rs` (`ts` not advanced within a `Sequence`) and `rules/layers.rs`
      (`toposort_with_timestamp_fallback`'s stable-sort tiebreak, "do not replace with an unstable
      sort").
    - **OOS seed** (per instruction, NOT fixed here): `ts` in `effects/mod.rs` is read but never
      advanced per `ApplyContinuousEffect` call. This is safe WITHIN one `Sequence` (relies on
      stable sort + push order, as verified above), but if two *separate* resolutions (e.g. two
      different spells/abilities resolving back-to-back with no intervening timestamp-advancing
      event) ever produced `ApplyContinuousEffect`s that landed on the exact same
      `state.timestamp_counter` value, their relative order would ALSO fall back to
      insertion-order-in-the-Vec rather than true CR 613.7 timestamp ordering — which is only
      correct by accident (VecDeque push order happens to track resolution order today, but nothing
      structurally guarantees that ordering claim for cross-resolution ties). Flagging as
      **OOS-AC7-3** for future primitive-batch investigation (does any resolution path fail to
      advance `state.timestamp_counter`, and could two resolutions ever collide on the same value?)
      — out of scope for this fix, which only needed to verify/lock in the SINGLE-resolution case.
  - F-VR2 (LOW): −2 clause otherwise oracle-accurate. `SetCardTypes({Artifact})` (preserves
    supertypes per ruling "It will retain any supertypes it had"), `LoseAllSubtypes` +
    `AddSubtypes({Treasure})` (ruling: "will lose any other subtypes … only a Treasure artifact"),
    no color change (correct — oracle/ruling do not change color). `TargetCreature` matches
    "Target creature." `EffectDuration::Indefinite` correct.
  - F-VR3 (LOW): −9 ability omitted with a valid TODO (OOS-AC7-1). Verified genuine gap: there is
    NO `Effect` that gives a player poison counters (poison is produced only by infect damage and
    Proliferate — effects/mod.rs), and no `EffectAmount` for "9 minus current poison." Gap is real.
  - F-VR4 (LOW): Compleated is unmodeled (def comment lines 30-31). Verified: no
    `KeywordAbility::Compleated` exists in the engine (only appears as comment/oracle-reminder text
    in vraska + ajani defs), so it cannot even be added as a marker. Oracle_text field DOES include
    the Compleated reminder text, so the text is accurate; only the loyalty-reduction behavior is
    absent. Pre-existing partial aspect, not part of the −2 clause under review.

---

## Integration test review (pb_ac7_card_integration.rs)
- No test asserts anything the oracle does not say; no vacuous passes observed.
  - Kenrith: asserts P/T 3/3, card_types {Creature}, subtypes {Elk}, green, no keywords, AND
    Legendary supertype preserved — all match oracle/rulings.
  - Eaten: asserts P/T 1/1, {Creature}/{Skeleton}, black, NOT red, no keywords — matches.
  - Darksteel: asserts P/T 0/1, {Artifact,Creature}/{Insect}, has Indestructible, no Flying —
    matches, and correctly guards the CR 613.7 grant-survives-removal ordering.
  - Sram: OR-semantics across Aura/Equipment/Vehicle (3 positive) + vanilla-creature negative —
    matches oracle.
  - Leaf-Crowned: static +1/+1 on other Elf (2/2), may-pay draw on Elf spell, negative on non-Elf
    — matches oracle.
  - Vraska −2: **added 2026-07-09** (F-VR1 fix) — asserts `Artifact`/`Treasure`, Legendary
    supertype preserved, no keywords/activated/triggered abilities, exactly one surviving mana
    ability with the correct shape. Non-vacuousness proven by a push-order perturbation (see
    F-VR1 entry above).
- Note: Final Showdown mode 0 is intentionally NOT covered here (its now-expressible clause reuses
  a pattern already covered elsewhere; see the card entry above).

## Summary
- Cards with issues: Vraska (1 MEDIUM verification gap — RESOLVED 2026-07-09 — + 3 LOW), Final
  Showdown (2 LOW, both documented/acceptable partials).
- Clean cards: Kenrith's Transformation, Eaten by Piranhas, Darksteel Mutation, Sram Senior
  Edificer, Leaf-Crowned Visionary (all fully oracle-accurate, zero markers).
- No HIGH findings. No stale/overbroad markers. No silent approximations.
