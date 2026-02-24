# Rules Gotchas — Last verified: Engine Core Complete checkpoint (2026-02-23)

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
