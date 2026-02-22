# M8 Session Plan — Replacement & Prevention Effects

Generated during M8 kickoff (2026-02-22). Milestone spec: `docs/mtg-engine-roadmap.md` M8 section.

## What M8 Delivers

- Replacement effect framework: intercept events before they fire, apply modifications
- Self-replacement priority (CR 614.15)
- Player choice when multiple replacements apply (CR 616.1)
- Loop prevention: a replacement can apply to a given event at most once (CR 614.5)
- Prevention effects: "prevent the next N damage" / "prevent all damage" (CR 615)
- Prevention + replacement interaction (CR 616)
- "If ~ would die" replacements → critical foundation for M9 commander zone-change
- "If a player would draw" replacements
- "Enters the battlefield" replacements (ETB tapped, ETB with counters)
- Commander zone-change choice wired up

## Architecture Summary

### New files
- `state/replacement_effect.rs` — `ReplacementId`, `ReplacementEffect`, `ReplacementTrigger`,
  `ReplacementModification`; register in `state/mod.rs`
- `rules/replacement.rs` — `apply_replacements()`, loop prevention, self-replacement priority;
  register in `rules/mod.rs`

### Data model
```
ReplacementEffect {
    id: ReplacementId,
    source: Option<ObjectId>,       // None = from a spell resolution (no permanent source)
    controller: PlayerId,
    duration: EffectDuration,       // reuse from continuous_effect.rs
    is_self_replacement: bool,      // CR 614.15 — applies before player-ordered effects
    trigger: ReplacementTrigger,
    modification: ReplacementModification,
}

ReplacementTrigger:
  WouldChangeZone { from: Option<ZoneType>, to: ZoneType, filter: ObjectFilter }
  WouldDraw { player_filter: PlayerFilter }
  WouldEnterBattlefield { filter: ObjectFilter }
  WouldGainLife { player_filter: PlayerFilter }
  DamageWouldBeDealt { target_filter: TargetFilter }

ReplacementModification:
  RedirectToZone(ZoneType)           // "to exile instead", "to command zone instead"
  EntersTapped                       // ETB replacement: permanent arrives tapped
  EntersWithCounters { counter, n }  // ETB replacement: arrives with N counters
  SkipDraw                           // "skip that draw"
  PreventDamage(u32)                 // prevent exactly N damage (decrements shield)
  PreventAllDamage                   // prevent all damage from this event
  ReplaceGainLifeWithDraw            // "draw that many cards instead"
```

### GameState changes
- Add `replacement_effects: Vec<ReplacementEffect>` to `GameState`
- Add `next_replacement_id: u64` counter
- `GameStateBuilder::with_replacement_effect(re) -> Self`

### New events (add to `rules/events.rs`)
```
ReplacementEffectApplied { effect_id: ReplacementId, description: String }
ReplacementChoiceRequired { player: PlayerId, event_description: String,
                             choices: Vec<ReplacementId> }
DamagePrevented { source: ObjectId, target: ..., prevented: u32, remaining: u32 }
```

### New command (add to `rules/command.rs` or equivalent)
```
Command::OrderReplacements { ids_in_order: Vec<ReplacementId> }
```

### AbilityDefinition
Add `Replacement { trigger: ReplacementTrigger, modification: ReplacementModification,
                   is_self: bool }` variant — static replacement abilities of permanents.

When a permanent with a `Replacement` ability enters the battlefield, the engine registers
a `ReplacementEffect` in `state.replacement_effects` (source = that object, duration =
`WhileSourceOnBattlefield`). When it leaves, the effect is filtered out at the same point
continuous effects are checked.

### Interception sites
1. **SBA zone changes** (`rules/sba.rs`): before emitting `CreatureDied` /
   `PlaneswalkerDied`, call `apply_replacements()` with `WouldChangeZone { to: Graveyard }`.
2. **Effect zone changes** (`effects/mod.rs`): `ExileObject`, `DestroyPermanent`,
   `ObjectPutInGraveyard` effects — check replacements before the move.
3. **Draw** (`rules/turn_actions.rs` draw step + `effects/mod.rs` DrawCards effect) —
   check `WouldDraw` replacements.
4. **ETB** (permanent enters battlefield in `rules/resolution.rs` and `effects/mod.rs`) —
   check `WouldEnterBattlefield` to apply tapped/counters modifications before the object
   settles on the battlefield.
5. **Damage** (`effects/mod.rs` DealDamage + `rules/combat.rs` combat damage) — check
   prevention and damage replacement effects.

## Session Breakdown

---

### Session 1 — Data model and GameState wiring
**~5 items**

Files: `state/replacement_effect.rs` (new), `state/mod.rs`, `state/builder.rs`

1. Define `ReplacementId(u64)`, `ReplacementEffect`, `ReplacementTrigger`,
   `ReplacementModification` (as outlined above). Full serde + clone + debug derives.
2. Add `ObjectFilter` enum (reuse/adapt from existing targeting): `AnyObject`,
   `CardId(CardId)`, `ControlledBy(PlayerId)`, `AnyCreature`, `Commander`.
3. Add `PlayerFilter` enum: `AnyPlayer`, `SpecificPlayer(PlayerId)`.
4. Add `replacement_effects: im::Vector<ReplacementEffect>` and
   `next_replacement_id: u64` to `GameState`.
5. Add `GameStateBuilder::with_replacement_effect(re: ReplacementEffect) -> Self` helper.
6. Tests: serialize/deserialize a `ReplacementEffect`, builder round-trip.

---

### Session 2 — Core application framework
**~6 items**

Files: `rules/replacement.rs` (new), `rules/mod.rs`, `rules/events.rs`,
`rules/command.rs` (or wherever `Command` lives)

1. `find_applicable(state, trigger: &ReplacementTrigger) -> Vec<ReplacementId>` —
   returns IDs of all currently-active replacement effects matching the trigger.
   (Active = source still on battlefield if `WhileSourceOnBattlefield`, duration
   not expired if `UntilEndOfTurn`.)
2. `apply_one(state, id: ReplacementId, events: &mut Vec<GameEvent>) -> GameState` —
   applies a single replacement effect, emitting `ReplacementEffectApplied`.
3. `apply_replacements(state, trigger, candidate_events, already_applied: HashSet<ReplacementId>)`
   — full loop:
   a. Filter out already-applied effects (CR 614.5).
   b. Sort self-replacements first (CR 614.15).
   c. If 0 applicable: return unmodified event.
   d. If 1 applicable: auto-apply.
   e. If 2+: emit `ReplacementChoiceRequired` and return `NeedsChoice` sentinel;
      the pending choice blocks further processing until `Command::OrderReplacements` arrives.
4. Add `ReplacementChoiceRequired`, `ReplacementEffectApplied`, `DamagePrevented`
   to `GameEvent`.
5. Add `Command::OrderReplacements { ids: Vec<ReplacementId> }` and wire into
   `process_command`.
6. Tests: 0 effects (no-op), 1 effect auto-applied, self-replacement sorted first,
   loop prevention (same effect can't apply twice), `OrderReplacements` command routes correctly.

---

### Session 3 — Zone-change interception + Commander die replacement
**~6 items**

Files: `rules/sba.rs`, `effects/mod.rs`, `cards/definitions.rs` (or wherever
commander-specific logic lives)

1. Wire `apply_replacements` into `sba.rs` creature-dies check:
   before moving creature to graveyard (before emitting `CreatureDied`), call with
   `WouldChangeZone { from: Some(Battlefield), to: Graveyard, filter: matching creature }`.
   If a replacement fires, redirect accordingly and emit `ReplacementEffectApplied`.
2. Wire into `sba.rs` planeswalker-dies check similarly.
3. Wire into `effects/mod.rs` `ExileObject` effect — check `WouldChangeZone { to: Exile }`.
4. Wire into `effects/mod.rs` `DestroyPermanent` effect — check `WouldChangeZone { to: Graveyard }`.
5. Commander zone-change replacement: when a commander would go to graveyard or exile,
   register a `ReplacementEffect { trigger: WouldChangeZone { to: Graveyard | Exile,
   filter: Commander }, modification: RedirectToZone(CommandZone), is_self: false }`.
   This is registered at game start for each commander (duration: `Indefinite`, source: None).
   Implement the choice prompt: `ReplacementChoiceRequired` for the owning player to choose
   command zone vs. the other replacement (or default zone).
6. Tests (per roadmap):
   - Commander dies → choice event emitted, choosing command zone works
   - Commander dies with Rest in Peace active → controller chooses order (case 18 from corner-cases)
   - Simple replacement: creature dies with no replacement → goes to graveyard normally
   - Creature dies with "exile instead" replacement → goes to exile

---

### Session 4 — Draw replacement + ETB replacement
**~5 items**

Files: `rules/turn_actions.rs`, `effects/mod.rs` (DrawCards effect), `rules/resolution.rs`
or wherever permanents enter the battlefield

1. Wire `apply_replacements` into the draw-card action (both turn-based draw step and
   the `DrawCards` effect): check `WouldDraw { player_filter }` before moving top library
   card to hand.
2. Implement `SkipDraw` modification: player skips the draw entirely (e.g., Teferi's
   Puzzle Box replacement where cards go back and are drawn fresh — simplified to skip
   for now).
3. Wire `apply_replacements` into the permanent-enters-battlefield path: after the object
   is placed in the Battlefield zone but before `PermanentEnteredBattlefield` is emitted,
   apply `WouldEnterBattlefield` replacements (e.g., `EntersTapped`, `EntersWithCounters`).
4. Update card definitions: lands with "enters the battlefield tapped" (Guildgates, etc.)
   should register `Replacement { trigger: WouldEnterBattlefield { filter: SelfObject },
   modification: EntersTapped, is_self: true }` as a static ability.
5. Tests (per roadmap):
   - Simple draw replacement (no applicable effect → card drawn normally)
   - "Skip that draw" replacement fires → no `CardDrawn` event
   - ETB tapped: land enters tapped, `PermanentTapped` fires, triggers watching "untapped"
     do NOT fire (case 19 from corner-cases)
   - ETB with counters: permanent arrives with correct counter count

---

### Session 5 — Prevention effects
**~5 items**

Files: `state/replacement_effect.rs` (add prevention shield state),
`rules/replacement.rs` (prevention logic), `rules/combat.rs`, `effects/mod.rs`

1. Prevention shields: add `PreventionShield { id: ReplacementId, remaining: u32 }` state
   or implement as a `ReplacementModification::PreventDamage(u32)` with mutable state
   tracked in `GameState` (decrement on each application). The `remaining` field lives on
   a separate `PreventionCounter` map in `GameState` keyed by `ReplacementId`.
2. Wire prevention into non-combat `DealDamage` effect: before applying damage, call
   `apply_replacements` with `DamageWouldBeDealt` trigger. `PreventDamage(n)` decrements
   the shield and reduces damage; `PreventAllDamage` zeroes it. Emit `DamagePrevented`.
3. Wire prevention into `apply_combat_damage` in `rules/combat.rs` similarly.
4. CR 616 ordering (prevention + replacement): when both a prevention and a non-prevention
   replacement apply to the same damage event, treat them both as applicable and let the
   player order them via `ReplacementChoiceRequired`.
5. Tests (per roadmap):
   - "Prevent the next 3 damage" shield: take 5 → 2 gets through, 3 prevented
   - Shield depletes correctly (state decrements)
   - "Prevent all damage" — all damage zeroed
   - Replacement + prevention interaction: player chooses order, both outcomes correct

---

### Session 6 — Card definitions + game scripts
**~6 items**

Files: `cards/definitions.rs`, `test-data/generated-scripts/replacement/` (new scripts)

1. Add/update card definitions that use replacement effects:
   - Rest in Peace (`WouldChangeZone { to: Graveyard }` → `RedirectToZone(Exile)`)
   - Leyline of the Void (same trigger, same modification, for opponents)
   - At least one ETB-tapped land definition if not already done in S4
   - At least one "if ~ would die" self-replacement card
   - Notion Thief (`WouldDraw` for opponents → skip + controller draws instead) — if
     `ReplaceGainLifeWithDraw` analogue is feasible; skip if too complex
2. Generate game scripts for `test-data/generated-scripts/replacement/`:
   - `replacement-001-simple.json`: single replacement effect, no player choice needed
   - `replacement-002-multiple-ordered.json`: two replacements, player chooses order (case 16)
   - `replacement-003-self-first.json`: self-replacement applies before other (case 17)
   - `replacement-004-commander-die.json`: commander dies, owner sends to command zone (case 18)
   - `replacement-005-etb-tapped.json`: land with "enters tapped" (case 19)
   - `replacement-006-kalitas.json`: commander dies with Kalitas-style exile replacement (case 28)
3. Human-review all scripts and mark `approved`.
4. Run full test suite: `cargo test --all` — all passing.
5. Run `cargo clippy -- -D warnings` — no warnings.
6. Run `cargo fmt --check` — clean.

---

## Milestone Acceptance Criteria Checklist

From roadmap M8:
- [ ] Replacement effects integrate cleanly with existing event system
- [ ] Commander zone-change choice works correctly
- [ ] No infinite loops possible in replacement effect chains
- [ ] Replacement effect game scripts pass through replay harness
- [ ] All 6 sessions complete
- [ ] 336+ tests passing (target: ~30 new tests from M8)

## Key CR References

- CR 614 — Replacement and Prevention Effects (general)
- CR 614.5 — A replacement effect can apply to a given event at most once
- CR 614.15 — Self-replacement effects apply first
- CR 615 — Prevention Effects
- CR 616 — Interaction of replacement + prevention effects
- CR 903.9 — Commander zone-change replacement

## Corner Cases Addressed

- Case 16: Multiple replacement effects, player chooses order → Session 2 + 3
- Case 17: Self-replacement applies first → Session 2
- Case 18: Commander dies with Rest in Peace → Session 3
- Case 19: ETB tapped, not a trigger → Session 4
- Case 28: Commander dies with Kalitas exile replacement → Session 3 + 6
- Case 33: Sylvan Library draw replacement (Dredge) → framework in Session 4; full
  Sylvan Library card definition deferred (complex multi-draw tracking)
