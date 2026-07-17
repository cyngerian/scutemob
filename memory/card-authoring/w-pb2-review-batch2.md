# Card Review: W-PB2 Batch 2 (dynamic-amount re-authoring)

**Reviewed**: 2026-07-17
**Cards**: 12 (11 Complete, 1 partial)
**Findings**: 1 HIGH, 0 MEDIUM, 4 LOW
**Verdicts**: 11 PASS, 1 FIX

## Engine facts established (used across cards)

- `effects::matches_filter` (effects/mod.rs:7941) **honors** `min_power`/`max_power`,
  `has_card_type`, `colors` (7962: `chars.colors.iter().any(|c| colors.contains(c))`),
  `has_subtype`, `has_subtypes` (OR), `has_card_types` (OR, 8035). So the decoy-critical
  filters in this batch (regal_force `colors=Green`, dockside `has_card_types=[Artifact,
  Enchantment]`, shamanic_revelation `min_power=4`, Elf subtype filters) are all load-bearing.
- `matches_filter` **does NOT** honor `exclude_self`; the `controller` field on `TargetFilter`
  is also invisible to it (Characteristics carries no controller). For `PermanentCount`, the
  player restriction is enforced separately by the outer `controller: PlayerTarget` via
  `players.contains(&obj.controller)` (effects/mod.rs:6752-6760) — so the redundant
  `controller: TargetController::You` inside each filter is harmless.
- `EffectAmount::PermanentCount` (6749-6771) has **no self-exclusion**. Only
  `AttackingCreatureCount` (7032) and `TappedCreatureCount` (7066) apply
  `(!f.exclude_self || obj.id != ctx.source)`. **This is the Éomer bug.**
- `SourcePowerAtLastKnownInformation` (7003) reads `ctx.lki_power`. For a `WhenDies` trigger
  the pre-death power is threaded pre_death_power → `PendingTrigger.lki_power`
  (abilities.rs:4200/4230) → `stack_obj.lki_power` (abilities.rs:7797) → `ctx.lki_power`
  (resolution.rs:2119/2209). Elenda cards resolve correctly.
- `ManaValueOf` (6686) reads via `state.fizzle_object(id)` (LKI after the spell has left the
  stack) and sums w+u+b+r+g+colorless+generic. Works for a countered spell in the graveyard.

---

## Card 1: Avenger of Zendikar — PASS
- Oracle/types/mana/P-T: YES ({5}{G}{G}, Creature—Elemental 5/5).
- ETB Plant count = `PermanentCount{filter: has_card_type=Land, controller You}` — "each land
  you control" (all lands, not just basics). Correct — matches oracle exactly (line 36-42).
- Landfall trigger = `WheneverPermanentEntersBattlefield{filter: Land+You, exclude_self:false}`
  → fires on your-lands-entering (line 65-72). The Avenger itself is an Elemental, not a land,
  so exclude_self is moot. ForEach over `Creature+Plant+You` adds a +1/+1 counter to each Plant
  (line 73-85). Fires and targets Plants correctly.
- **F1 (LOW)**: oracle "you **may** put a +1/+1 counter" is modeled unconditionally (mandatory
  take), per the khalni_heart_expedition convention cited in-file (line 60-62). Always
  beneficial; acceptable per project convention. Documented, not a correctness regression.

## Card 2: Elven Ambush — PASS
- Oracle/types/mana: YES ({3}{G} Instant).
- Token = 1/1 green Elf Warrior; count = `PermanentCount{Creature+Elf subtype, You}` = "each Elf
  you control" (line 28-36). Correct. Subtype filter is honored by matches_filter.

## Card 3: Elvish Promenade — PASS
- Oracle/mana: YES. **Type line correct**: `types_sub(&[Kindred, Sorcery], &["Elf"])` = "Kindred
  Sorcery — Elf" (line 14) — the Kindred (Tribal) card type is present as required.
- Token + count identical to Elven Ambush. Correct.

## Card 4: Regal Force — PASS
- Oracle/types/mana/P-T: YES ({4}{G}{G}{G} Elemental 5/5).
- Draw count = `PermanentCount{filter: Creature + colors=Some({Green}), You}` (line 24-31).
  **Decoy verified**: `colors=Green` is load-bearing — matches_filter:7962 rejects a non-green
  creature you control (its `chars.colors` shares no member with {Green}). Non-green creatures
  are correctly NOT counted. Matches oracle "each green creature you control".

## Card 5: Shaman of the Pack — PASS (correctly `partial`)
- Oracle/types/mana/P-T: YES ({1}{B}{G} Elf Shaman 3/2).
- ETB "target opponent loses life = number of Elves you control" is left unimplemented.
  **Marker is honest**: `TargetRequirement` (card_definition.rs:2760) has NO opponent-restricted
  variant; `TargetPlayer` is "any player" and permits an illegal self-target. Shipping it would
  be wrong game state per W5 — omission is correct. The LoseLife *amount* is expressible
  (`PermanentCount{Elf subtype, You}`), exactly as the completeness note states, so the blocker
  is genuinely the target restriction only.
- **F2 (LOW)**: the inline TODO comment (lines 31-33) is stale/self-contradicting — it claims
  "Needs `EffectAmount::SubtypeCount("Elf", You)` — not in DSL", but the completeness note (and
  reality) is that `PermanentCount` already expresses the amount. Doc inconsistency only; no
  behavior. Recommend aligning the inline comment with the completeness note.

## Card 6: Dockside Extortionist — PASS
- Oracle/types/mana/P-T: YES ({1}{R} Goblin Pirate 1/2).
- Treasure count = `PermanentCount{filter: has_card_types=[Artifact,Enchantment],
  controller EachOpponent}` (line 26-31). **Decoy verified**: `has_card_types` OR-match
  (matches_filter:8035) counts opponents' permanents that are artifact **or** enchantment (an
  artifact enchantment counts once, correct); outer `EachOpponent` restricts to opponents. Matches
  oracle "artifacts and enchantments your opponents control".
- Token via `..treasure_token_spec(1)` with `count` given explicitly first — the explicit field
  overrides the spread's default of 1 (valid Rust struct-update precedence). Correct.

## Card 7: Cavern-Hoard Dragon — PASS
- Oracle/types/mana/P-T: YES ({7}{R}{R} Dragon 6/6). Flying/Trample/Haste keywords present.
- Cost reduction = `SelfCostReduction::MaxOpponentPermanents{filter: Artifact, per:1}` — models
  "greatest number of artifacts an opponent controls" (max, not sum), matching the oracle and the
  in-file CR 601.2f note (line 26-34).
- Combat-damage trigger: `WhenDealsCombatDamageToPlayer` → Treasure count =
  `PermanentCount{filter: Artifact, controller DamagedPlayer}` with outer
  `controller: PlayerTarget::DamagedPlayer` (line 47-54). Correctly resolves to "each artifact
  **that player** controls" (the damaged player, not each opponent). Distinct from Dockside — the
  right player scope.

## Card 8: Shamanic Revelation — PASS
- Oracle/types/mana: YES ({3}{G}{G} Sorcery).
- Draw = `PermanentCount{Creature, You}` = "each creature you control" (line 24-31). Correct.
- Ferocious = `ForEach` over `{Creature, You, min_power:4}` with `GainLife{Controller,
  Fixed(4)}` per iteration (line 35-46). **Verified per-creature, not flat**: sums to 4×N where N
  = creatures with power ≥ 4. matches_filter:7947 honors `min_power`. Matches oracle "gain 4 life
  for each creature you control with power 4 or greater".

## Card 9: Access Denied — PASS
- Oracle/types/mana: YES ({3}{U}{U} Instant).
- Sequence: `CounterSpell{DeclaredTarget 0}` then `CreateToken` with
  `count: ManaValueOf(DeclaredTarget 0)` (line 24-49). `ManaValueOf` reads via `fizzle_object`
  (LKI), which still resolves after CounterSpell has moved the spell off the stack. Token is
  1/1 colorless (empty colors) Artifact Creature — Thopter with flying — matches oracle exactly.
  (Brief's mention of a "Servo" token is a slip; oracle/def are Thopter, and correct.)
- **F3 (LOW)**: `ManaValueOf` sums the fizzled object's printed `mana_cost` fields and does not
  add the chosen X. Countering an {X} spell (e.g. an X-cost hydra cast for X=5) would create
  tokens equal to the printed cost only, not the on-stack mana value including X (CR 202.3b).
  Narrow edge (only X-spells); pre-existing DSL limitation of `ManaValueOf`, not introduced here.

## Card 10: Elenda, the Dusk Rose — PASS
- Oracle/types/mana/P-T: YES (Legendary Vampire Knight 1/1, Lifelink). Supertype Legendary present.
- "Whenever another creature dies" = `WheneverCreatureDies{controller:None, exclude_self:true}`
  (line 34-38) — any creature, excludes self; correct (this is the trigger path, where
  exclude_self IS honored — unrelated to the PermanentCount gap). Adds +1/+1 to Elenda.
- Dies trigger: `WhenDies` → CreateToken count `SourcePowerAtLastKnownInformation` = Elenda's
  power at death (including accumulated counters), via the LKI thread confirmed above. Token =
  1/1 white Vampire **with lifelink** (keywords:[Lifelink], line 67). Matches oracle.

## Card 11: Elenda's Hierophant — PASS
- Oracle/types/mana/P-T: YES (Vampire Cleric 1/1, Flying).
- "Whenever you gain life" → +1/+1 on self (line 27-33). Correct.
- Dies trigger token count = `SourcePowerAtLastKnownInformation` (line 55); token = 1/1 white
  Vampire **with lifelink** (line 57) — required per oracle, present. Mirrors Elenda. Correct.

## Card 12: Éomer, King of Rohan — FIX (HIGH)
- Oracle/types/mana/P-T: YES (Legendary Human Noble 2/2, Double strike). Supertype present.
- "enters with a +1/+1 counter for each **other** Human you control" is correctly modeled as a
  **replacement** (`ReplacementModification::EntersWithCounters`, CR 614.1c) with
  `count: PermanentCount{filter: Creature+Human+You, exclude_self:true}` (line 39-51) — the DSL
  is written correctly. ETB trigger (monarch + damage-equal-to-power to any target) with
  `[TargetPlayer, TargetAny]` is correct.
- **F4 (HIGH)** — **wrong game state**: `exclude_self:true` is **silently ignored** by the
  `PermanentCount` resolver. The self-ETB replacement runs at replacement.rs:1607-1653 with
  `ctx.source = Éomer` (replacement.rs:1622-1623), but by then Éomer is already on the
  battlefield — it was moved there at resolution.rs:576-577, well before
  `apply_self_etb_from_definition` at resolution.rs:1646. The count block (effects/mod.rs:6749-
  6771) calls `matches_filter`, which has no `exclude_self` branch, and the block itself lacks the
  `obj.id != ctx.source` guard that `AttackingCreatureCount`/`TappedCreatureCount` carry
  (7032/7066). **Éomer therefore counts itself as a Human**, entering with (other Humans + 1)
  counters — a bare 2/2 with no other Humans enters as a 3/3. The card is marked `Complete` but
  produces wrong P/T.
  - **Fix (engine, preferred)**: add `&& (!filter.exclude_self || obj.id != ctx.source)` to the
    `PermanentCount` filter closure at effects/mod.rs:6749-6771, mirroring lines 7032/7066. This
    is a general gap — any Complete card using `PermanentCount` with `exclude_self` is affected;
    Éomer is the instance in this batch. After the fix the def needs no change.
  - **Alternative (card)**: demote to `known_wrong` until the resolver honors exclude_self. Do not
    leave `Complete` as-is.

---

## Summary
- **FIX (1)**: eomer_king_of_rohan (HIGH — `PermanentCount` ignores `exclude_self`; self-counts
  as a Human, enters with one too many +1/+1 counters). Recommend engine fix at
  effects/mod.rs:6749-6771.
- **Clean / PASS (11)**: avenger_of_zendikar, elven_ambush, elvish_promenade, regal_force,
  shaman_of_the_pack (honest `partial`), dockside_extortionist, cavern_hoard_dragon,
  shamanic_revelation, access_denied, elenda_the_dusk_rose, elendas_hierophant.
- **LOW notes** (non-blocking): avenger "may"→mandatory (convention-OK); shaman stale inline
  TODO comment vs. completeness note; access_denied `ManaValueOf` ignores chosen X on X-spells.
- No gated stubs (`Choose`/`MayPayOrElse`/`AddManaChoice`/`AddManaAnyColor`) found in any Complete
  def in this batch.
