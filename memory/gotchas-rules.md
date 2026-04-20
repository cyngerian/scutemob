# Rules Gotchas — Last verified: M9.5 + PB-T (2026-04-20)

## MTG Rules Gotchas

- **Object identity (CR 400.7)**: When an object changes zones, it becomes a NEW object.
  The old ObjectId is dead. Auras fall off. "When this dies" triggers reference the old
  object. This is the #1 source of bugs in MTG engines.
- **Replacement effects are NOT triggers.** They modify events as they happen. They don't
  use the stack. Getting this wrong breaks the entire event system.
- **Storm copies are NOT cast** (CR 702.40c). "Whenever you cast" triggers do NOT fire for
  storm copies. Storm copies also cannot themselves trigger storm. Cascade works the same way —
  the free cast from cascade IS cast (triggers fire), but copies made by storm are not.
- **Protection prevents DEBT from matching sources** (CR 702.16): Damage, Enchanting/Equipping,
  Blocking, Targeting. It does NOT stop non-targeted effects ("destroy all creatures" hits
  protected creatures). The single interception point in the engine is `apply_damage_prevention`
  as the first check.
- **SBAs are checked as a batch, not individually.** All applicable SBAs happen simultaneously.
  Then triggers from all of them go on the stack together (in APNAP order).
- **Layer dependency check must handle circular dependencies.** CR 613.8k says fall back to
  timestamp order. If your dependency resolver can infinite-loop, it will.
- **"Commander damage" only counts COMBAT damage.** Not regular damage. A copy of a commander
  does NOT count — the copy isn't a commander.
- **Tokens cease to exist when they leave the battlefield** — but they DO briefly exist in
  the new zone first (long enough to trigger "when this dies" etc.).

- **Dredge is a replacement effect, not a trigger or activated ability (CR 702.52a).** It
  modifies the "would draw" event using the existing `check_would_draw_replacement` infrastructure.
  Wire it into BOTH the draw-step path (`turn_actions.rs`) and the effect-draw path
  (`effects/mod.rs`). Dredge does NOT increment `cards_drawn_this_turn` — it is a replacement,
  not a draw. `draw_card_skipping_dredge` is a helper that bypasses the replacement check to
  avoid re-offering the choice after the player declines.
- **Flashback must exile at ALL departure points (CR 702.34a).** The card must be exiled when
  it leaves the stack for ANY reason: (1) normal resolution, (2) fizzle (all targets illegal),
  (3) countered by a spell/ability, AND (4) the `CounterSpell` effect path in `effects/mod.rs`.
  Missing any one of the 4 paths causes the card to go to the graveyard instead of exile.
- **Cycling is instant-speed (CR 702.29a).** No sorcery restriction — cycling can be activated
  any time you could cast an instant. The discard is the cost (paid before the draw ability
  hits the stack); the draw ability can be countered. Do not add `TimingRestriction::SorcerySpeed`
  to cycling activated abilities.
- **Aura WhenDies triggers fire on `AuraFellOff`, not `CreatureDied` (CR 704.5m).** When an
  Aura is put into the graveyard by SBA (because its enchanted permanent left the battlefield),
  the engine emits `GameEvent::AuraFellOff`. The `check_triggers` function in `abilities.rs`
  must have an explicit arm for `AuraFellOff` — the `CreatureDied` arm does NOT cover it.
  Missing this arm causes WhenDies triggers on Auras (e.g., Rancor's return-to-hand trigger)
  to silently never fire.
- **Multi-target validators cannot greedily match slots in declaration order (CR 601.2c).**
  "Up to N target X" and any `Vec<TargetRequirement>` validator must accept a legal target
  in *any* declared slot, not just the first one it syntactically fits. The correct shape is
  two-pass best-fit: pass 1 collects candidates per slot, pass 2 does a bipartite match.
  Greedy-consume-in-order wrongly rejects CR-legal declarations (e.g., player picks target A
  for slot 2 and skips slot 1 — a greedy validator sees slot 1 as "no target, fail" before
  it reaches slot 2). Discovered during PB-T review; fixed in `casting.rs` validator.
- **Enchant is enforced at TWO checkpoints (CR 702.5b, 303.4a, 303.4c, 303.4d, 704.5m):**
  (1) **Cast-time** (`casting.rs`): the named target must already be on the battlefield AND
  match the `EnchantTarget` type. (2) **SBA** (`sba.rs`): if the Aura's target is no longer
  legal (left battlefield, changed type, or Aura is enchanting itself — CR 303.4d), the Aura
  goes to the graveyard. Use a typed `EnchantTarget` enum (not a boolean `enchants_creatures`)
  so SBA helpers can re-check legality for any target type. Aura attachment (setting
  `attached_to` / `attachments`) must happen BEFORE `register_static_continuous_effects` in
  `resolution.rs` — otherwise the Aura's static effects fire before the attachment is recorded
  and they will find no target to modify.
- **Convoke, Delve, Improvise are NOT additional costs (CR 702.51b, 702.66b, 702.126a).** They
  apply AFTER the total cost is determined (including commander tax, flashback alternative cost,
  etc.). The cost-modifier pipeline order in `casting.rs`: base mana cost → flashback alt cost →
  commander tax → kicker → convoke → improvise → delve → payment. Any future cost-reduction
  keyword must insert after tax/before payment.
  **Improvise reduces generic mana only** (unlike Convoke which reduces any color). Each tapped
  artifact reduces the generic cost by 1. Validate: artifact is on battlefield, controlled by
  caster, untapped, IS an artifact, no duplicates, count ≤ generic remaining.
- **Convoke does NOT require summoning sickness exemption** (ruling under CR 702.51a). Tapping
  a creature for convoke is not an activated ability tap cost — summoning sickness only prevents
  `{T}` activated abilities. A creature that entered this turn can still convoke.
- **Toxic (702.164) is STATIC; Poisonous (702.70) is TRIGGERED.** Toxic applies inline as a
  combat damage result (CR 120.3g) — no trigger on the stack, cannot be responded to separately,
  multiple instances are cumulative (sum all N values). Poisonous IS a triggered ability — goes on
  stack, can be countered, each instance triggers separately. Both give poison counters *in addition
  to* normal life loss (unlike Infect which replaces). Toxic is enforced in `combat.rs`
  `apply_combat_damage_assignments()`; Poisonous uses the trigger dispatch system (like Ingest).
- **Ninjutsu does NOT fire "whenever this creature attacks" (CR 508.3a/508.4).** The ninja is
  "put onto the battlefield tapped and attacking" — it never went through declare_attackers, so no
  AttackersDeclared event includes it. ETB triggers DO fire normally. The ninja inherits the
  returned creature's attack target specifically (CR 702.49c) — not a general choose-any rule.
  Commander Ninjutsu from command zone does NOT increment commander_tax (ruling 2020-11-10).
- **Batch 3 CR corrections (another batch-plan gotcha):** Melee=702.121 (702.122=Crew),
  Enlist=702.154 (702.155=different), Toxic=702.164 (702.156=Ravenous). Always MCP-verify.
- **Combat damage trigger infrastructure: `check_triggers` must fire on TBA events (CR 510.3a).**
  `enter_step` processes turn-based actions (like assigning combat damage). Triggers from those
  TBAs must be checked inside `enter_step` itself — `check_triggers()` called only after player
  commands will miss triggers that originate in step-entry TBAs.
- **CR text overrides card rulings — always.** Card rulings (Gatherer/Scryfall) are dated
  annotations written at print time. When the CR changes, old rulings become stale and are
  never retroactively updated. Example: the June 2025 CR 714.4 update changed Blood Moon +
  Urza's Saga behavior, but the 2021-06-18 Scryfall ruling still says the opposite.
  **Derive implementation from the CR. Use rulings only to identify edge cases to test.**
- **Blood Moon has effects in BOTH Layer 4 and Layer 6.** Layer 4: type-change to Mountain
  (loses other land subtypes). Layer 6: removes ALL abilities (including non-subtype-based
  ones like Urza's Saga's chapter abilities) and adds `{T}: Add {R}`. Do not implement Blood
  Moon as a Layer 4-only effect and assume ability removal follows implicitly — it must be
  modeled with an explicit Layer 6 component too.
- **Blood Moon vs Alpine Moon in Layer 6**: Alpine Moon explicitly says "loses all abilities"
  (a clear Layer 6 effect). Blood Moon's ability removal also applies in Layer 6 but comes
  from its "are Mountains" type-change. The distinction matters for timestamp ordering against
  other Layer 6 effects (e.g., Saga gained abilities).
- **Blood Moon applies in Layer 4 (type-changing); gained abilities from resolved chapter
  abilities apply in Layer 6.** Because Layer 6 comes after Layer 4, abilities that Urza's
  Saga *gained* from resolved chapter abilities (e.g. `{T}: Add {C}` from Chapter I) survive
  Blood Moon — Blood Moon's type-change strips the printed chapter abilities but doesn't
  override the Layer 6 gained-ability effects. Alpine Moon explicitly says "lose all
  abilities" in Layer 6, so it *does* remove gained abilities (assuming it entered after).
- **Dress Down triggers the same Saga SBA behavior as Blood Moon.** Any effect that removes
  all abilities from a Saga (Dress Down, etc.) invokes the CR 714.4 "one or more chapter
  abilities" check. The Saga is not sacrificed. This is not Blood Moon-specific — the SBA
  must check the condition generically for any ability-removal effect.
- **OPEN QUESTION — lore counter addition under ability removal**: Does a Saga that has lost
  all its abilities still receive lore counters at the beginning of main phase? Depends on
  whether the lore counter addition is an intrinsic rule of the Saga *subtype* (CR 714) or a
  printed ability that gets removed. The article states no more counters are added under Blood
  Moon, but verify against CR 714 before implementing.

- **"As ~ enters the battlefield, [choose X / it becomes Y]" is a REPLACEMENT effect, not a triggered ability (CR 614.12).** PB-X C1 bug: Obelisk of Urd was authored as `AbilityDefinition::Triggered { trigger_condition: WhenEntersBattlefield, effect: ChooseCreatureType }`. This is wrong. The choice must resolve atomically with ETB, before any priority window and before any other ETB trigger sees the permanent's battlefield state. Authoring as a trigger leaves the permanent on the battlefield with `chosen_creature_type = None` during the trigger-resolution window; the pump is inactive then, and responses at that point violate CR 614.12. Correct form: `AbilityDefinition::Replacement { trigger: WouldEnterBattlefield, modification: ChooseCreatureType(default), is_self: true }`. In-codebase templates: Urza's Incubator, Vanquisher's Banner, Morophon, Cavern of Souls, Patchwork Banner, Heralds' Horn. When authoring any "As this enters, ..." card, grep one of these first.

- **Mana-production replacement: "you" vs "its controller" are different scopes.** Caged Sun says "causes **you** to add" — replacement fires only when Caged Sun's controller taps a land. Gauntlet of Power says "**its controller** adds" — replacement should fire for *any* player who taps a basic of the chosen color. The engine's `ManaWouldBeProduced { controller }` currently only implements "you" (= replacement source's controller) semantics. "Its controller" (= tapping player regardless of who controls the replacement source) requires a new scope field on the replacement trigger (PB-Q3). Discovered during PB-Q review; `mana.rs:310` filter is structurally correct for "you" cards, structurally wrong for "its controller" cards. When authoring mana-production replacement cards, always check whether the oracle text says "you" or names a different controller.

---

## Top-10 Corner Cases

(6 M8-direct + 4 general — full details in `docs/mtg-engine-corner-cases.md`)

### #16 — Multiple replacement effects, player chooses order (CR 616.1)
If two or more replacements apply to the same event, the affected player/controller chooses
the order. Each applies once. If the result is again affected by a remaining replacement,
apply it immediately. Watch for: each replacement applies to the modified event, not the
original.

### #17 — Self-replacement effects apply first (CR 614.15)
An effect saying "if X would happen to [this object], instead..." has priority over
replacements from other sources. Order among multiple self-replacements: affected
player/controller chooses.

### #18 — Commander zone-change (SBA) + Rest in Peace (replacement)
Commander graveyard/exile redirect is an **SBA** (CR 903.9a), not a replacement effect.
After a commander dies or is exiled, the SBA check moves it to the command zone. If Rest
in Peace also applies (exile replacement), both fire: RiP replaces the "dies → graveyard"
with "exile"; then the SBA fires and moves it from exile to command zone. Hand/library
redirects (CR 903.9b) ARE replacement effects (the rule says "instead").

### #19 — "Enters tapped" replacement (CR 614.1c)
"Enters the battlefield tapped" is a replacement effect on the ETB event, not a triggered
ability. The permanent was NEVER untapped on the battlefield — it didn't "enter untapped
then tap." Matters for abilities that trigger on "entering untapped."

### #28 — Commander dies + Kalitas (replacement vs SBA)
Kalitas replaces "creature dies" with "exile it and create a token" (replacement effect).
Commander graveyard redirect is an SBA (CR 903.9a), not a replacement. So: if Kalitas
applies, commander is exiled; then the SBA fires and owner may move it from exile to
command zone. If Kalitas does NOT apply (commander goes to graveyard), the SBA fires
and moves it to command zone. No competing replacements — they operate at different times.

### #33 — Sylvan Library + replaced draws
Sylvan Library tracks cards drawn in the draw step. If a draw is replaced by an effect
that doesn't use the word "draw," those cards don't count for Sylvan Library. Only
replacements that still result in drawing count.

### #1 — Humility + Opalescence (CR 613.10)
Both affect each other. Timestamp order matters. If Humility entered first: Opalescence
makes it a creature (L4) → Humility removes all abilities including its own (L6) →
Humility's P/T setting (L7b) no longer applies to itself. Both become 1/1 creatures with
no abilities.

### #8 — Deathtouch + Trample (CR 702.19b, 702.78b)
For trample, attacker must assign "lethal damage" to each blocker before assigning to the
player. With deathtouch, lethal = 1. A 5/5 deathtouch+trample blocked by a 2/2 can assign
1 to the blocker and 4 to the player.

### #24 — Tokens briefly in non-battlefield zones (CR 704.5d)
When a token leaves the battlefield, it briefly exists in the new zone — triggering "when
this dies" etc. — then ceases to exist as an SBA. Effects like Kalitas that exile before
the graveyard DO prevent the "dies" trigger.

### #34 — Mandatory infinite loops (CR 726)
If a loop involves only mandatory actions (no player choices), it must continue indefinitely
or the game draws. If it involves optional actions, the active player must stop it if it
benefits no player or only the active player. Engine needs non-termination detection.

### #35 — CDA (Characteristic-Defining Abilities) apply in all zones (CR 604.3)
CDAs like Devoid and Changeling function everywhere — hand, graveyard, exile, stack, even
outside the game. When implementing a CDA, do NOT add a battlefield-only guard in
`calculate_characteristics()`. The layer system iterates all objects regardless of zone.

### #37 — Linked abilities fire even when the keyword is removed (CR 607.2a)
Abilities linked by "exile with it" or "was exiled with" (Champion, Soulbond, etc.) still
fire even if the keyword ability has been removed by a continuous effect. The detection
check must use the *state field* (e.g., `champion_exiled_card.is_some()`) — never guard
with `KeywordAbility::X` presence. Champion's LTB arm was caught doing this wrong.

### #38 — Creature-equipment/fortification rules (CR 301.5b, 301.6 analog)
A permanent that is both a Fortification and a creature cannot fortify a land (CR 301.6).
Similarly equipment-creatures can't equip. Both checks must use layer-resolved types
(`calculate_characteristics`), not base `card_types`, because the permanent may have been
turned into a creature by a continuous effect applied after printing.

### #36 — Ability-batch-plan.md CR numbers can be wrong
The batch plan was authored before MCP lookups. Planners have caught multiple errors:
- Horsemanship: plan said 702.30 (wrong — that's Echo); correct is 702.31
- Skulk: plan said 702.120 (wrong — that's Escalate); correct is 702.118
- Decayed: plan said 702.145 (wrong); correct is 702.147
Always verify CR numbers via MCP lookup at plan time, not from the batch plan table.

### #39 — Subtype-filter test wedges must operate on the filter's target property
When writing a test that exercises a filter-read dispatch site (e.g., "subtype-filtered
death trigger"), the test wedge must discriminate on **the property the filter actually
reads**, not on an incidental field that happens to differ pre- vs post-condition.

**Originating incident**: PB-N F3. Test 6 was originally named for LKI death evaluation
but used `Color::Black` as the wedge — a static field with no continuous effect ending
at death. The test passed under both pre- and post-death evaluation because the color
was never variable. The filter read subtype, not color; the wedge should have been on
subtype.

**Rule**: for any `filter` dispatch test, the wedge property must be:
1. The exact property the filter reads (subtype for subtype filter, color for color
   filter, card_type for card_type filter, etc.)
2. Variable in a way that differs between the pre-condition and post-condition being
   tested (on-battlefield vs. in-graveyard, pre-death vs. post-death, etc.)
3. Independently observable (you can verify with `assert!` that the variation is
   present in the pre-condition before running the assertion on the post-condition)

If your proposed wedge doesn't meet all three, your test is the silent-skip pattern
recurring. Pick a different property or a different mechanism.

**Known structural limitation (BASELINE-LKI-01)**: the LKI dispatch at
`rules/abilities.rs:4180-4202` currently re-runs layer filters against the graveyard
object via `calculate_characteristics`, so any wedge that depends on a filter-matched
continuous effect ending at zone change is structurally unobservable. Both
`LayerModification + SingleObject` and `LayerModification + AttachedCreature` grants
were verified to fail this way in the PB-N fix phase experiment. Do not try to build a
wedge around these until BASELINE-LKI-01 is fixed; use a different test pattern
(e.g., base characteristics + filter-read dispatch, which at least validates the
dispatch path consumption of the filter field, without validating the LKI semantics).
