# W-PB2 Batch 5 Review — modal/UpToN targeting, alt-cost, misc

**Reviewed**: 2026-07-17
**Cards**: 9 (8 Complete, 1 partial)
**Findings**: 0 HIGH, 0 MEDIUM, 4 LOW
**Verdict**: 9 PASS (simian_spirit_guide PASS-as-partial). No FIX/DEMOTE.

## Engine facts verified (load-bearing for this batch)
- **Out-of-range `DeclaredTarget` no-ops, never panics.** `resolve_effect_target_list_indexed`
  (`effects/mod.rs:6124-6162`): `ctx.targets.get(idx)` returns `None` for an unfilled UpToN
  slot → `vec![]`, and even a filled slot skips if the object no longer exists (partial
  fizzle, CR 608.2b). So cloud_of_faeries / frantic_search / rewind untap DT indices past the
  chosen count safely.
- **`Effect::DrainLife`** (`effects/mod.rs:544-574`): iterates `PlayerTarget::EachOpponent`,
  each loses `amount`, `total_lost` accumulated, controller gains the actual total (CR 702.101a).
  Exactly the Exsanguinate shape.
- **`Cost::ExileFromHand { color }`** (`card_definition.rs:1247`): confirmed to be the
  Force-of-Will pitch alt-cost (doc at 1241-1246: exiles a card of `color` to help cast a
  DIFFERENT spell, recorded on `CastSpell.additional_costs`). Not a self-exile. `Cost::DiscardSelf`
  (1233, Channel) exists but is discard-not-exile. No `ExileSelfFromHand` analog. Simian's
  stated blocker is accurate.

## Card 1: Cloud of Faeries — PASS (Complete)
- Oracle/types/mana: {1}{U}, Creature — Faerie, 1/1. Match.
- Flying + ETB untap-up-to-2 + Cycling {2} all present. Dual-def for Cycling correct
  (`Keyword(Cycling)` marker at line 47 + `Cycling { cost }` at 48-53, KI-6 satisfied).
- ETB: `UpToN{2, TargetLand}` (targets 40-43) + `Sequence[UntapPermanent{DT0}, UntapPermanent{DT1}]`
  (31-38). Fewer-than-2 chosen: DT1 no-ops (engine fact above). Correct.
- F1 (LOW): oracle_text omits Cycling reminder text "({2}, Discard this card: Draw a card.)"
  that MCP returns (line 17). Cosmetic; corpus is inconsistent on reminder text. Non-blocking.

## Card 2: Frantic Search — PASS (Complete)
- Oracle/types/mana: {2}{U}, Instant. Match.
- `Sequence[Draw2, Discard2, Untap DT0..DT2]` (17-37), targets `[UpToN{3,TargetLand}]` (38-41).
- **Discard IS present** (lines 22-25) — the old draw-with-no-discard bug is gone. Correct.

## Card 3: Insatiable Avarice — PASS (Complete)
- Oracle/types/mana: {B}, Sorcery, Spree. Match (base cost {B} at 10-13; Spree reminder text included).
- `Keyword(Spree)` marker (20) + `Spell` with `modes: Some(ModeSelection{ min 1, max 2 })`.
- **`Spell.targets` is empty** (25). Mode costs `[{2}, {B}{B}]` (33-42) match "+ {2}" / "+ {B}{B}".
- Mode 0 = SearchLibrary → Library Top, `shuffle_before_placing: true` (47-57) matches
  "search... then shuffle and put that card on top."
- Mode 1 = target player Draw3 then Lose3 (62-71); `mode_targets: Some([[], [TargetPlayer]])` (73).
  Per-mode-local DeclaredTarget index 0 (boros_charm precedent). Correct target player, correct order.

## Card 4: Niv-Mizzet, the Firemind — PASS (Complete)
- Types/mana/PT: {2}{U}{U}{R}{R} (generic 2/blue 2/red 2), Legendary Creature — Dragon Wizard,
  4/4. Legendary supertype present (17-21). Match.
- **Draw trigger is `DealDamage{DT0, 1}` + `targets:[TargetAny]`** (33-38) — NOT ForEach{EachOpponent}.
  The old "3× untargeted damage" multiplayer bug is absent. Correct.
- `{T}: Draw a card` activated (44-55), Flying (28). Correct.
- F2 (LOW): def writes "Niv-Mizzet, the Firemind deals 1 damage" (line 22) while MCP returns the
  abbreviated "Niv-Mizzet deals 1 damage". The def matches the full modern Scryfall oracle (self-
  reference by full name); the MCP string is the abbreviated/older form. Non-blocking; def is
  likely the more-correct text. Flag only to record the MCP delta.

## Card 5: Rewind — PASS (Complete)
- Oracle/types/mana: {2}{U}{U}, Instant. Match.
- targets `[TargetSpell, UpToN{4,TargetLand}]` (46-52) — **spell in slot 0**.
- `Sequence[CounterSpell{DT0}, Untap DT1..DT4]` (28-45). Slot indexing: mandatory TargetSpell
  consumes position 0, UpToN{4} consumes 1..4. Correct; unfilled land slots no-op.

## Card 6: Simian Spirit Guide — PASS as partial (correctly demoted)
- Types/mana/PT: {2}{R}, Creature — Ape Spirit, 2/2. Match.
- **The invented free "Add {R}" bug is REMOVED**: `abilities: vec![]` (29). The prior
  `Cost::Mana(default)` free-repeatable-untapped infinite-mana ability is gone.
- **Blocker is real and correctly named** (30-39): "exile this card from your hand" as an
  activation cost has no Cost variant. `Cost::ExileFromHand{color}` is verified to be the
  unrelated FoW pitch alt-cost (engine fact above); `Cost::DiscardSelf` is discard-not-exile.
  Partial marker cites the exact needed primitive (`Cost::ExileSelfFromHand` + activation_zone: Hand).
- Note (a): confirmed — `ExileFromHand{color}` is the pitch cost, not self-exile. Note (b):
  confirmed — the illegal free ability no longer registers. Marker is accurate. No action.

## Card 7: Stensia Masquerade — PASS (Complete)
- Oracle/types/mana: {2}{R}, Enchantment. Match.
- FirstStrike static on `AttackingCreaturesYouControl` (23-31) — unchanged, correct.
- Vampire combat trigger: `WheneverCreatureYouControlDealsCombatDamageToPlayer{ filter: Vampire }`
  → `AddCounter{TriggeringCreature, +1/+1}` (34-53). "on it" = triggering creature. Correct.
- **Madness dual-def present**: `Keyword(Madness)` marker (55) + `Madness { cost: {2}{R} }` (56-62).
  KI-6 satisfied.
- F3 (LOW): oracle_text omits Madness reminder text (line 19). Cosmetic. Non-blocking.

## Card 8: Exsanguinate — PASS (Complete)
- Oracle/types: Sorcery. Match.
- **Mana `{X}{B}{B}` → `ManaCost{ black: 2, x_count: 1 }`** (12-16). x_count IS present — the old
  mis-cost-as-{B}{B} bug is gone. Correct.
- `DrainLife{ XValue }` (21-23) is the each-opponent-loses-X / controller-gains-total shape
  (engine fact above, CR 702.101a). Correct. targets empty (untargeted). Correct.

## Card 9: Sylvan Messenger — PASS (Complete)
- Oracle/types/mana/PT: {3}{G}, Creature — Elf, 2/2. Match (Trample reminder text included).
- Trample keyword (30) + ETB `RevealAndRoute{ count 4, filter Elf, matched→Hand, unmatched→
  Library{Bottom} }` (34-48). Filter subtype Elf, matched=Hand, unmatched=Library Bottom — all
  correct. goblin_ringleader precedent for fixed-order bottom. Correct.

## Gated-stub scan
No `Choose` / `MayPayOrElse` / `AddManaChoice` / `AddManaAnyColor` in any of the 8 Complete defs.
`Effect::Sequence`, `RevealAndRoute`, `DrainLife`, `SearchLibrary`, `ModeSelection`, `UpToN`,
`CounterSpell` only. Clean.

## Summary
- Cards with issues: none functional. LOW/cosmetic only: cloud_of_faeries (F1), niv_mizzet (F2),
  stensia_masquerade (F3) — reminder-text/self-reference oracle deltas vs MCP.
- Clean/PASS: all 9. simian_spirit_guide correctly remains partial with an accurate blocker.
- No DEMOTE, no FIX required.
