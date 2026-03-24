# Primitive Batch Review: PB-26 -- Trigger Variants

**Date**: 2026-03-24
**Reviewer**: primitive-impl-reviewer (Opus)
**CR Rules**: CR 603.2 (trigger matching), CR 603.2c (one trigger per event), CR 603.2g (prevented/replaced events), CR 603.10a (LTB look-back), CR 701.9a (discard), CR 701.21a (sacrifice), CR 508.1 (declare attackers)
**Engine files reviewed**: `crates/engine/src/cards/card_definition.rs` (TriggerCondition variants), `crates/engine/src/state/game_object.rs` (TriggerEvent variants), `crates/engine/src/rules/events.rs` (PermanentSacrificed GameEvent), `crates/engine/src/rules/abilities.rs` (check_triggers dispatch, flush_pending_triggers), `crates/engine/src/effects/mod.rs` (SacrificePermanents emission, TriggeringPlayer resolution), `crates/engine/src/rules/casting.rs` (bargain/emerge/casualty emission), `crates/engine/src/rules/resolution.rs` (devour emission, champion sacrifice), `crates/engine/src/state/hash.rs` (discriminants), `crates/engine/src/testing/replay_harness.rs` (TC->TE mapping)
**Card defs reviewed**: 51 card definitions across all 8 gap types
**Tests reviewed**: `crates/engine/tests/trigger_variants.rs` (19 tests)

## Verdict: needs-fix

Three findings: one HIGH (CardDrawn dispatch missing triggering_player tagging, breaks Scrawling Crawler and Razorkin Needlehead), one MEDIUM (Champion sacrifice path missing PermanentSacrificed emission), one MEDIUM (Camellia the Seedmiser sacrifice trigger not updated despite primitive now existing). The rest of the implementation is correct. LTB look-back is properly wired on all 6 zone-change events. Spell-type filtering logic is correct. Hash discriminants are complete. All other card defs match oracle text for the abilities they implement, with appropriate TODOs documenting remaining DSL gaps.

## Engine Change Findings

| # | Severity | File:Line | Description |
|---|----------|-----------|-------------|
| 1 | **HIGH** | `abilities.rs:4869-4918` | **CardDrawn dispatch missing triggering_player.** Cards using `PlayerTarget::TriggeringPlayer` with draw triggers get wrong player. **Fix:** add tagging block. |
| 2 | MEDIUM | `resolution.rs:3802-3861` | **Champion sacrifice missing PermanentSacrificed.** Champion's "sacrifice it" path does not emit the event. **Fix:** add emission after zone move. |

## Card Definition Findings

| # | Severity | Card | Description |
|---|----------|------|-------------|
| 3 | MEDIUM | `camellia_the_seedmiser.rs` | **Sacrifice-Food trigger not implemented.** TODO says primitive missing, but `WheneverYouSacrifice` now exists. **Fix:** add trigger. |

### Finding Details

#### Finding 1: CardDrawn dispatch missing triggering_player tagging

**Severity**: HIGH
**File**: `crates/engine/src/rules/abilities.rs:4869-4918`
**CR Rule**: CR 603.2 -- "Whenever a game event or game state matches a triggered ability's trigger event, that ability automatically triggers."
**Issue**: The `GameEvent::CardDrawn` handler in `check_triggers` collects triggers for `ControllerDrawsCard`, `OpponentDrawsCard`, and `AnyPlayerDrawsCard` but never sets `triggering_player` on the collected `PendingTrigger` entries. The `CardDiscarded` handler (line 4986-4989) correctly tags triggers with `t.triggering_player = Some(*player)`, and the `PermanentSacrificed` handler (line 5036-5038) also correctly tags them. The `CardDrawn` handler is the only one that omits this.

This means `PlayerTarget::TriggeringPlayer` -- which resolves via `flush_pending_triggers` storing `triggering_player` as `Target::Player` at index 0 -- will have no player to resolve. Cards using this pattern:
- `scrawling_crawler.rs`: "Whenever an opponent draws a card, that player loses 1 life" uses `PlayerTarget::TriggeringPlayer`
- `razorkin_needlehead.rs`: "Whenever an opponent draws a card, this creature deals 1 damage to them" uses `PlayerTarget::TriggeringPlayer`

Without the tag, `TriggeringPlayer` falls through to `ctx.player_for_target(0)` (returns None since no target was set), then `ctx.triggering_player` (also None), then the final fallback `ctx.controller` -- meaning the controller of Scrawling Crawler/Razorkin Needlehead loses life/takes damage instead of the opponent who drew. This is a wrong game state.

**Fix**: After the `AnyPlayerDrawsCard` collection block (after line 4917), add:
```rust
// Tag draw triggers with the drawing player for PlayerTarget::TriggeringPlayer.
for t in &mut triggers[pre_len..] {
    t.triggering_player = Some(*player);
}
```
where `pre_len` is captured before the ControllerDrawsCard collection begins (same pattern as CardDiscarded at line 4986-4989). Move the existing `let pre_len = triggers.len();` to before line 4870.

#### Finding 2: Champion sacrifice path missing PermanentSacrificed emission

**Severity**: MEDIUM
**File**: `crates/engine/src/rules/resolution.rs:3802-3861`
**CR Rule**: CR 701.21a -- "To sacrifice a permanent, its controller moves it from the battlefield directly to its owner's graveyard."
**Oracle**: Champion (CR 702.72a) -- "sacrifice it unless you exile another [type]"
**Issue**: When a Champion creature fails to find a qualifying target and must be sacrificed (line 3802), the code emits `CreatureDied` or `ObjectExiled` but does NOT emit `GameEvent::PermanentSacrificed`. All other sacrifice paths in the engine (effects/mod.rs, abilities.rs sacrifice_self/sacrifice_filter, casting.rs bargain/emerge/casualty, resolution.rs devour) correctly emit this event. Champion's sacrifice is semantically a sacrifice per CR 702.72a ("sacrifice it"), so it should trigger "whenever you sacrifice a permanent" abilities like Korvold and Juri.

**Fix**: In the Champion sacrifice resolution code (resolution.rs ~line 3802-3861), after emitting `CreatureDied` or `ObjectExiled` for each zone-change action (Redirect/Proceed), also emit:
```rust
events.push(GameEvent::PermanentSacrificed {
    player: pre_sacrifice_controller,
    object_id: source_object,
    new_id: new_grave_id, // or new_id for the redirect case
});
```

#### Finding 3: Camellia the Seedmiser sacrifice trigger not updated

**Severity**: MEDIUM
**File**: `crates/engine/src/cards/defs/camellia_the_seedmiser.rs:33-35`
**Oracle**: "Whenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token."
**Issue**: The card def still has a TODO comment saying "Requires TriggerCondition::WheneverYouSacrificeFood (not yet implemented). Deferred until sacrifice-trigger infrastructure is added." However, PB-26 added `TriggerCondition::WheneverYouSacrifice { filter: Some(TargetFilter { has_subtype: Some(SubType("Food")) }), player_filter: None }` which is exactly what Camellia needs. Other cards like Captain Lannery Storm (Treasure filter) and Tireless Tracker (Clue filter) were correctly updated to use this pattern. Camellia was missed.

Note: Oracle says "one or more Foods" (batched trigger), but the DSL fires per-sacrifice. This is a known engine-wide approximation (per-event triggers, not batched).

**Fix**: Replace lines 33-35 with:
```rust
AbilityDefinition::Triggered {
    trigger_condition: TriggerCondition::WheneverYouSacrifice {
        filter: Some(TargetFilter {
            has_subtype: Some(SubType("Food".to_string())),
            ..Default::default()
        }),
        player_filter: None,
    },
    effect: Effect::CreateToken {
        spec: TokenSpec {
            name: "Squirrel".to_string(),
            card_types: [CardType::Creature].into_iter().collect(),
            subtypes: [SubType("Squirrel".to_string())].into_iter().collect(),
            colors: [Color::Green].into_iter().collect(),
            power: 1,
            toughness: 1,
            count: 1,
            supertypes: im::OrdSet::new(),
            keywords: im::OrdSet::new(),
            tapped: false,
            enters_attacking: false,
            mana_color: None,
            mana_abilities: vec![],
            activated_abilities: vec![],
        },
    },
    intervening_if: None,
    targets: vec![],
},
```

## CR Coverage Check

| CR Rule | Implemented? | Tested? | Notes |
|---------|-------------|---------|-------|
| CR 603.2 (trigger matching) | Yes | Yes | test_whenever_you_cast_creature_spell_filter_stored, and 18 other tests |
| CR 603.2c (one trigger per event) | Yes | Yes | test_whenever_you_attack_fires_once_per_combat |
| CR 603.10a (LTB look-back) | Yes | Yes | test_when_leaves_battlefield_trigger_variant, _fires_on_death, _fires_on_destruction |
| CR 701.9a (discard) | Yes | Yes | test_whenever_you_discard_trigger_variant, test_whenever_opponent_discards_trigger_variant |
| CR 701.21a (sacrifice) | Yes | Yes | test_whenever_you_sacrifice_trigger_variant, _with_filter, _not_on_destruction |
| CR 508.1 (declare attackers) | Yes | Yes | test_whenever_you_attack_trigger_variant, _fires_once_per_combat |
| CR 603.2g (prevented events) | Not tested | No | No negative test for prevented sacrifice/discard |

## Card Def Summary

| Card | Oracle Match | TODOs Remaining | Game State Correct | Notes |
|------|-------------|-----------------|-------------------|-------|
| talrand_sky_summoner | Yes | 0 | Yes | Instant/sorcery filter correct |
| guttersnipe | Yes | 0 | Yes | Instant/sorcery filter correct |
| murmuring_mystic | Yes | 0 | Yes | Instant/sorcery filter correct |
| archmage_emeritus | Yes | 1 (copy) | Yes | "or copy" gap noted |
| beast_whisperer | Yes | 0 | Yes | Creature filter correct |
| monastery_mentor | Yes | 0 | Yes | noncreature_only correct, Prowess keyword present |
| whispering_wizard | Yes | 1 (once/turn) | Yes | "once each turn" noted |
| lys_alana_huntmaster | Yes | 2 (may, subtype) | Yes | Creature approximation for Elf |
| sram_senior_edificer | Yes | 1 (subtype) | No* | Unfiltered trigger fires on all spells, not Aura/Equipment/Vehicle |
| bontus_monument | Yes | 1 (cost reduce) | Yes | Creature filter correct |
| oketras_monument | Yes | 1 (cost reduce) | Yes | Creature filter correct |
| hazorets_monument | Yes | 2 (cost, may) | No* | Draws without discard prerequisite |
| nezahal_primal_tide | Yes | 2 (uncounterable, no max hand) | Yes | noncreature_only on OpponentCastsSpell correct |
| mystic_remora | Yes | 1 (may pay) | Yes | noncreature_only correct, MayPayOrElse present |
| archmage_of_runes | Yes | 1 (cost reduce) | Yes | Instant/sorcery filter correct |
| slickshot_show_off | Yes | 0 | Yes | noncreature_only correct, Plot present |
| hermes_overseer_of_elpis | Yes | 1 (Bird attack) | Yes | noncreature_only correct |
| leaf_crowned_visionary | Yes | 2 (may, subtype) | Yes | Creature approximation for Elf |
| storm_kiln_artist | Yes | 1 (CDA) | Yes | Instant/sorcery filter correct |
| chulane_teller_of_tales | Yes | 2 (land drop, bounce) | Yes | Creature filter, draw only (land drop gap) |
| inexorable_tide | Yes | 0 | Yes | No filter (any spell), Proliferate correct |
| alela_cunning_conqueror | Yes | 2 (first/turn, Faerie) | Yes | during_opponent_turn correct |
| rhystic_study | Yes | 0 | Yes | No filter on OpponentCastsSpell |
| waste_not | Yes | 1 (card type filter) | No* | Only Zombie trigger, missing land/noncreature-nonland |
| lilianas_caress | Yes | 0 | Yes | TriggeringPlayer correct |
| megrim | Yes | 0 | Yes* | Uses LoseLife not DealDamage (approximation noted) |
| raiders_wake | Yes | 1 (Raid) | Yes | Discard trigger correct |
| fell_specter | Yes | 0 | Yes | ETB + discard trigger both correct |
| glint_horn_buccaneer | Yes | 1 (activated) | Yes | Discard trigger correct |
| brallin_skyshark_rider | Yes | 1 (Shark grant) | Yes | Discard + counter + damage correct |
| korvold_fae_cursed_king | Yes | 1 (forced sac) | Yes | Sacrifice trigger correct |
| smothering_abomination | Yes | 1 (forced sac) | Yes | Creature filter on sacrifice correct |
| captain_lannery_storm | Yes | 0 | Yes | Treasure subtype filter correct |
| tireless_tracker | Yes | 0 | Yes | Clue subtype filter correct |
| juri_master_of_the_revue | Yes | 1 (death damage) | Yes | Sacrifice trigger correct |
| mirkwood_bats | Yes | 1 (token-only) | No* | Fires on all sacrifices not just tokens |
| carmen_cruel_skymarcher | Yes | 1 (attack GY return) | Yes | player_filter: Any correct |
| mayhem_devil | Yes | 1 (any target) | No* | Deals to each opponent not "any target" |
| camellia_the_seedmiser | Yes | 1 (Food sac) | No* | **Missing sacrifice trigger (Finding 3)** |
| caesar_legions_emperor | Yes | 1 (modal reflexive) | No* | Creates tokens unconditionally (no sacrifice required) |
| seasoned_dungeoneer | Yes | 1 (protection+explore) | No* | VentureIntoDungeon approximation instead of protection+explore |
| chivalric_alliance | Yes | 1 (two+ creatures) | No* | Fires on any attack, not "two or more creatures" |
| mishra_claimed_by_gix | Yes | 2 (X=attackers, meld) | No* | Fixed(1) instead of attacking creature count |
| anim_pakal_thousandth_moon | Yes | 1 (counter-based tokens) | No* | Only adds counter, missing token creation |
| clavileno_first_of_the_blessed | Yes | 1 (type change+grant) | No* | Empty abilities (deferred) |
| aven_riftwatcher | Yes | 0 | Yes | ETB + LTB gain 2 life correct |
| toothy_imaginary_friend | Yes | 0 | Yes | Draw + LTB draw-per-counter correct |
| sengir_autocrat | Yes | 1 (token-only exile) | Yes | ETB + LTB exile Serfs correct |
| elder_deep_fiend | Yes | 1 (multi-target) | Yes | WhenYouCastThisSpell correct, single target approximation |
| niv_mizzet_the_firemind | Yes | 1 (any target) | No* | Deals to each opponent not "any target" |
| consecrated_sphinx | Yes | 1 (may) | Yes | Opponent draw filter correct |
| smothering_tithe | Yes | 1 (may pay) | Yes | Opponent draw filter correct |
| scrawling_crawler | Yes | 0 | **No** | **TriggeringPlayer broken (Finding 1)** |
| razorkin_needlehead | Yes | 0 | **No** | **TriggeringPlayer broken (Finding 1)** |
| vito_thorn_of_the_dusk_rose | Yes | 1 (amount) | No* | Fixed(1) not "that much life"; EachOpponent not target |
| elendas_hierophant | Yes | 1 (death tokens) | Yes | Lifegain trigger correct |
| marauding_blight_priest | Yes | 0 | Yes | Lifegain trigger dispatch wiring fixed it |

*Items marked No* have pre-existing approximation issues documented with TODOs, not introduced by PB-26.

## Test Coverage Assessment

The 19 tests cover:
- G-4: spell_type_filter stored/fires correctly (3 tests)
- G-9: discard triggers fire for controller and opponent (2 tests)
- G-10: sacrifice trigger fires, filter works, does not fire on destruction (3 tests)
- G-11: attack trigger fires once per combat (2 tests)
- G-12: LTB triggers fire on death, destruction, and exile (3 tests -- no bounce test)
- G-13: draw trigger fires for controller (2 tests), opponent draw filter (1 test)
- G-14: lifegain trigger fires (1 test)
- G-15: cast-this-spell trigger variant stored and fires from stack (2 tests)

Missing coverage:
- No test for LTB trigger on bounce (ObjectReturnedToHand)
- No test for LTB trigger on sacrifice (PermanentSacrificed)
- No test for `PlayerTarget::TriggeringPlayer` resolution with draw triggers (would catch Finding 1)
- No test for sacrifice filter on opponents' sacrifices (player_filter: Any with filter)
