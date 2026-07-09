//! PB-AC6 card integration tests — Searslicer Goblin, Chart a Course, Bloodsoaked
//! Champion, Idol of Oblivion, Dark Petition, Land Tax.
//!
//! These exercise the *real* `CardDefinition`s in `crates/engine/src/cards/defs/`
//! (not synthetic fixtures) through full `process_command` flows, validating that
//! the PB-AC6 primitives (`Condition::YouAttackedThisTurn`,
//! `Condition::CreatedATokenThisTurn`, `Condition::SpellMastery`,
//! `Condition::OpponentControlsMoreLandsThanYou`) are wired correctly into each
//! card's abilities.
//!
//! Venerated Rotpriest is intentionally excluded: its trigger condition is now
//! expressible, but its effect ("target opponent gets a poison counter") has no
//! corresponding `Effect` variant, so the ability stays `ENGINE-BLOCKED` and there
//! is nothing card-specific to integration-test beyond the already-covered Toxic
//! keyword.

use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::state::CardType;
use mtg_engine::{
    enrich_spec_from_def, process_command, CardDefinition, CardId, CardRegistry, Command, Effect,
    EffectAmount, GameEvent, GameState, GameStateBuilder, ObjectId, ObjectSpec, PlayerId, Step,
    SuperType, TokenSpec, ZoneId,
};
use std::collections::HashMap;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

/// Build the name→definition map that `enrich_spec_from_def` expects.
fn defs_of(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

/// `ObjectSpec::card()` produces a *naked* object: no mana cost, no abilities,
/// no P/T. Without enrichment a cast pays nothing (`chars.mana_cost` is `None`,
/// so `mana_value() == 0` short-circuits payment) and `ActivateAbility` reports
/// `InvalidAbilityIndex`. Every test below that casts or activates a real card
/// must route its spec through here.
fn card_spec(
    player: PlayerId,
    name: &str,
    card_id: &str,
    zone: ZoneId,
    def: &CardDefinition,
) -> ObjectSpec {
    enrich_spec_from_def(ObjectSpec::card(player, name), &defs_of(def))
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(zone)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found", name))
}

fn find_in_zone(state: &GameState, name: &str, zone: ZoneId) -> Option<ObjectId> {
    state
        .objects
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name && obj.zone == zone)
        .map(|(id, _)| *id)
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Hand(player))
        .count()
}

/// Pass priority for all listed players once (resolves top of stack or advances turn).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", pl, e));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Pass priority repeatedly until `target` step is reached. Mirrors the
/// `advance_to_step` convention used across the test suite (e.g.
/// `pb_ac6_phase_action_conditions.rs`).
fn advance_to_step(mut state: GameState, target: Step) -> GameState {
    let mut guard = 0;
    loop {
        if state.turn.step == target {
            return state;
        }
        guard += 1;
        assert!(
            guard < 500,
            "advance_to_step exceeded safety guard (infinite loop?)"
        );
        let holder = state.turn.priority_holder.expect("no priority holder");
        let (new_state, _) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = new_state;
    }
}

/// Resolve everything currently on the stack by passing priority in turn order.
/// Reaching a step does NOT resolve the triggers that step queued — they go on
/// the stack (CR 603.3) and need priority passes.
fn resolve_stack(mut state: GameState, players: &[PlayerId]) -> GameState {
    let mut guard = 0;
    while !state.stack_objects.is_empty() {
        guard += 1;
        assert!(guard < 100, "resolve_stack exceeded safety guard");
        state = pass_all(state, players).0;
    }
    state
}

fn empty_cast_spell(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell {
        player,
        card,
        targets: vec![],
        convoke_creatures: vec![],
        improvise_artifacts: vec![],
        delve_cards: vec![],
        kicker_times: 0,
        alt_cost: None,
        prototype: false,
        modes_chosen: vec![],
        x_value: 0,
        face_down_kind: None,
        additional_costs: vec![],
        hybrid_choices: vec![],
        phyrexian_life_payments: vec![],
    }
}

// ── 1. Searslicer Goblin — Raid, CR 508.1 ───────────────────────────────────────

#[test]
/// CR 508.1 (Raid) / oracle: "At the beginning of your end step, if you attacked
/// this turn, create a 1/1 red Goblin creature token." Verifies the token is
/// created when an attacker was declared this turn, and NOT created when no
/// attacker was declared.
fn test_searslicer_goblin_raid_creates_token_only_if_attacked() {
    let def = mtg_engine::cards::defs::searslicer_goblin::card();
    let registry = CardRegistry::new(vec![def.clone()]);
    let p1 = p(1);
    let p2 = p(2);

    // -- Attacked this turn: token IS created. --
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry.clone())
            .object(card_spec(
                p1,
                "Searslicer Goblin",
                "searslicer-goblin",
                ZoneId::Battlefield,
                &def,
            ))
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();

        let attacker_id = find_object(&state, "Searslicer Goblin");
        let (state, _) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
            },
        )
        .unwrap();

        // CR 603.3: reaching the end step only *queues* the trigger onto the stack;
        // it must be resolved by priority passes before the token exists.
        let state = advance_to_step(state, Step::End);
        let state = resolve_stack(state, &[p1, p2]);
        let token = state.objects.iter().find(|(_, obj)| {
            obj.characteristics.name == "Goblin" && obj.zone == ZoneId::Battlefield
        });
        assert!(
            token.is_some(),
            "Raid: a 1/1 red Goblin token must be created at end step when the \
             controller attacked this turn"
        );
        let (_, tok_obj) = token.unwrap();
        assert_eq!(tok_obj.characteristics.power, Some(1));
        assert_eq!(tok_obj.characteristics.toughness, Some(1));
    }

    // -- Did NOT attack this turn: no token created. --
    {
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Searslicer Goblin",
                "searslicer-goblin",
                ZoneId::Battlefield,
                &def,
            ))
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();

        // No DeclareAttackers command issued -- attacked_this_turn stays false.
        let state = advance_to_step(state, Step::End);
        let state = resolve_stack(state, &[p1, p2]);
        let token = state.objects.iter().find(|(_, obj)| {
            obj.characteristics.name == "Goblin" && obj.zone == ZoneId::Battlefield
        });
        assert!(
            token.is_none(),
            "Raid: no Goblin token should be created when the controller did not \
             attack this turn"
        );
    }
}

// ── 2. Chart a Course ────────────────────────────────────────────────────────

#[test]
/// Oracle: "Draw two cards. Then discard a card unless you attacked this turn."
/// Verifies both branches of the `Condition::YouAttackedThisTurn`-gated
/// `Effect::Conditional`: discard occurs when the flag is false, and is skipped
/// (both drawn cards kept) when the flag is true.
fn test_chart_a_course_discards_unless_attacked() {
    let def = mtg_engine::cards::defs::chart_a_course::card();

    let build = |attacked: bool| {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let mut b = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Chart a Course",
                "chart-a-course",
                ZoneId::Hand(p1),
                &def,
            ));
        for i in 0..3 {
            b = b.object(
                ObjectSpec::creature(p1, &format!("Library Card {i}"), 1, 1)
                    .in_zone(ZoneId::Library(p1)),
            );
        }
        let mut state = b
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;
        state.players.get_mut(&p1).unwrap().mana_pool.blue = 1;
        state.turn.priority_holder = Some(p1);
        if attacked {
            state.players.get_mut(&p1).unwrap().attacked_this_turn = true;
        }
        state
    };

    // -- Did not attack: draw 2, discard 1 -> net hand size 1. --
    {
        let state = build(false);
        let spell_id = find_object(&state, "Chart a Course");
        let (state, _) = process_command(state, empty_cast_spell(p1(), spell_id)).unwrap();
        let (state, _) = pass_all(state, &[p1(), p2()]);
        assert_eq!(
            hand_count(&state, p1()),
            1,
            "unless you attacked this turn: draw 2 then discard 1 nets a hand size of 1"
        );
        // The resolved sorcery itself also lands in the graveyard (CR 608.2m), so
        // count only the discarded library card.
        assert_eq!(
            state
                .objects
                .iter()
                .filter(|(_, obj)| obj.zone == ZoneId::Graveyard(p1())
                    && obj.characteristics.name.starts_with("Library Card"))
                .count(),
            1,
            "the discarded card must be in the graveyard"
        );
    }

    // -- Attacked this turn: draw 2, no discard -> net hand size 2. --
    {
        let state = build(true);
        let spell_id = find_object(&state, "Chart a Course");
        let (state, _) = process_command(state, empty_cast_spell(p1(), spell_id)).unwrap();
        let (state, _) = pass_all(state, &[p1(), p2()]);
        assert_eq!(
            hand_count(&state, p1()),
            2,
            "having attacked this turn: draw 2 and keep both (no discard)"
        );
    }
}

fn p1() -> PlayerId {
    p(1)
}
fn p2() -> PlayerId {
    p(2)
}

// ── 3. Bloodsoaked Champion — Raid graveyard reanimation ────────────────────────

#[test]
/// CR 508.1 (Raid) / CR 602.2: "{1}{B}: Return this card from your graveyard to
/// the battlefield. Activate only if you attacked this turn." Verifies the
/// activated ability succeeds from the graveyard when the controller attacked
/// this turn, and is rejected (activation_condition unmet) when they did not.
fn test_bloodsoaked_champion_raid_reanimation_gated_by_attack() {
    let def = mtg_engine::cards::defs::bloodsoaked_champion::card();

    // -- Attacked this turn: activation succeeds, reanimates to the battlefield. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(ObjectSpec::creature(p1, "Raider", 2, 2))
            .object(card_spec(
                p1,
                "Bloodsoaked Champion",
                "bloodsoaked-champion",
                ZoneId::Graveyard(p1),
                &def,
            ))
            .active_player(p1)
            .at_step(Step::DeclareAttackers)
            .build()
            .unwrap();

        let attacker_id = find_object(&state, "Raider");
        let (mut state, _) = process_command(
            state,
            Command::DeclareAttackers {
                player: p1,
                attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
                enlist_choices: vec![],
                exert_choices: vec![],
            },
        )
        .unwrap();

        state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;
        state.players.get_mut(&p1).unwrap().mana_pool.black = 1;
        state.turn.priority_holder = Some(p1);

        let champion_id = find_object(&state, "Bloodsoaked Champion");
        let (state, _) = process_command(
            state,
            Command::ActivateAbility {
                player: p1,
                source: champion_id,
                ability_index: 0,
                targets: vec![],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        )
        .unwrap_or_else(|e| panic!("Raid activation should succeed when attacked: {:?}", e));
        let (state, _) = pass_all(state, &[p1, p2]);

        assert!(
            find_in_zone(&state, "Bloodsoaked Champion", ZoneId::Battlefield).is_some(),
            "CR 508.1: Bloodsoaked Champion must return to the battlefield when the \
             raid condition is met"
        );
    }

    // -- Did NOT attack this turn: activation is rejected. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Bloodsoaked Champion",
                "bloodsoaked-champion",
                ZoneId::Graveyard(p1),
                &def,
            ))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();

        state.players.get_mut(&p1).unwrap().mana_pool.colorless = 1;
        state.players.get_mut(&p1).unwrap().mana_pool.black = 1;
        state.turn.priority_holder = Some(p1);

        let champion_id = find_object(&state, "Bloodsoaked Champion");
        let result = process_command(
            state,
            Command::ActivateAbility {
                player: p1,
                source: champion_id,
                ability_index: 0,
                targets: vec![],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        );
        assert!(
            result.is_err(),
            "CR 508.1: raid activation must be rejected when the controller has not \
             attacked this turn"
        );
    }
}

// ── 4. Idol of Oblivion — activation gated by CreatedATokenThisTurn ────────────

#[test]
/// CR 111.10 / oracle: "{T}: Draw a card. Activate only if you created a token
/// this turn." Verifies the draw ability succeeds after a token was created this
/// turn, and is rejected when none was.
fn test_idol_of_oblivion_draw_gated_by_created_token() {
    let def = mtg_engine::cards::defs::idol_of_oblivion::card();

    // -- A token was created this turn: activation succeeds and draws a card. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Idol of Oblivion",
                "idol-of-oblivion",
                ZoneId::Battlefield,
                &def,
            ))
            .object(ObjectSpec::creature(p1, "Library Filler", 1, 1).in_zone(ZoneId::Library(p1)))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.turn.priority_holder = Some(p1);

        let dummy_source = ObjectId(99999);
        let mut ctx = EffectContext::new(p1, dummy_source, vec![]);
        let _ = execute_effect(
            &mut state,
            &Effect::CreateToken {
                spec: TokenSpec {
                    name: "Made Token".to_string(),
                    power: 1,
                    toughness: 1,
                    card_types: [CardType::Creature].into_iter().collect(),
                    count: EffectAmount::Fixed(1),
                    ..Default::default()
                },
            },
            &mut ctx,
        );

        let idol_id = find_object(&state, "Idol of Oblivion");
        let hand_before = hand_count(&state, p1);
        let (state, _) = process_command(
            state,
            Command::ActivateAbility {
                player: p1,
                source: idol_id,
                ability_index: 0,
                targets: vec![],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        )
        .unwrap_or_else(|e| {
            panic!(
                "Idol of Oblivion's draw ability should activate after creating a token: {:?}",
                e
            )
        });
        let (state, _) = pass_all(state, &[p1, p(2)]);
        assert_eq!(
            hand_count(&state, p1),
            hand_before + 1,
            "CR 111.10: activation must draw a card once a token was created this turn"
        );
    }

    // -- No token created this turn: activation is rejected. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p(2))
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Idol of Oblivion",
                "idol-of-oblivion",
                ZoneId::Battlefield,
                &def,
            ))
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.turn.priority_holder = Some(p1);

        let idol_id = find_object(&state, "Idol of Oblivion");
        let result = process_command(
            state,
            Command::ActivateAbility {
                player: p1,
                source: idol_id,
                ability_index: 0,
                targets: vec![],
                discard_card: None,
                sacrifice_target: None,
                x_value: None,
            },
        );
        // Must be rejected by the `CreatedATokenThisTurn` activation condition, not
        // because the ability failed to resolve at all — an unenriched spec would
        // yield `InvalidAbilityIndex` and pass this assertion for the wrong reason.
        match result {
            Err(mtg_engine::GameStateError::InvalidAbilityIndex { .. }) => panic!(
                "ability was not found on the object at all — the spec was not enriched, \
                 so this test would not have exercised the activation condition"
            ),
            Err(_) => {}
            Ok(_) => panic!(
                "CR 111.10: the draw ability must be rejected when no token was created \
                 this turn"
            ),
        }
    }
}

// ── 5. Dark Petition — Spell mastery bonus mana ─────────────────────────────────

#[test]
/// CR 207.2c (ability word) / oracle: "Spell mastery — If there are two or more
/// instant and/or sorcery cards in your graveyard, add {B}{B}{B}." Verifies the
/// bonus mana is added when spell mastery is active, and withheld when it is not
/// (base tutor effect happens either way).
fn test_dark_petition_spell_mastery_bonus_mana() {
    let def = mtg_engine::cards::defs::dark_petition::card();

    // -- Spell mastery active (2 instant/sorcery in graveyard): bonus BBB added. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Dark Petition",
                "dark-petition",
                ZoneId::Hand(p1),
                &def,
            ))
            .object(ObjectSpec::creature(p1, "Library Target", 1, 1).in_zone(ZoneId::Library(p1)))
            .object(
                ObjectSpec::card(p1, "GY Instant")
                    .with_types(vec![CardType::Instant])
                    .in_zone(ZoneId::Graveyard(p1)),
            )
            .object(
                ObjectSpec::card(p1, "GY Sorcery")
                    .with_types(vec![CardType::Sorcery])
                    .in_zone(ZoneId::Graveyard(p1)),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;
        state.players.get_mut(&p1).unwrap().mana_pool.black = 2;
        state.turn.priority_holder = Some(p1);

        let spell_id = find_object(&state, "Dark Petition");
        let (state, _) = process_command(state, empty_cast_spell(p1, spell_id)).unwrap();
        let (state, _) = pass_all(state, &[p1, p2]);

        assert!(
            find_in_zone(&state, "Library Target", ZoneId::Hand(p1)).is_some(),
            "base search effect must put the found card into hand"
        );
        assert_eq!(
            state.players.get(&p1).unwrap().mana_pool.black,
            3,
            "spell mastery: {{B}}{{B}}{{B}} must be added to the mana pool (pool was \
             fully spent on the cast, so the only black mana remaining is the bonus)"
        );
    }

    // -- Spell mastery inactive (only 1 instant/sorcery in graveyard): no bonus. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let mut state = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(card_spec(
                p1,
                "Dark Petition",
                "dark-petition",
                ZoneId::Hand(p1),
                &def,
            ))
            .object(ObjectSpec::creature(p1, "Library Target 2", 1, 1).in_zone(ZoneId::Library(p1)))
            .object(
                ObjectSpec::card(p1, "GY Instant 2")
                    .with_types(vec![CardType::Instant])
                    .in_zone(ZoneId::Graveyard(p1)),
            )
            .active_player(p1)
            .at_step(Step::PreCombatMain)
            .build()
            .unwrap();
        state.players.get_mut(&p1).unwrap().mana_pool.colorless = 3;
        state.players.get_mut(&p1).unwrap().mana_pool.black = 2;
        state.turn.priority_holder = Some(p1);

        let spell_id = find_object(&state, "Dark Petition");
        let (state, _) = process_command(state, empty_cast_spell(p1, spell_id)).unwrap();
        let (state, _) = pass_all(state, &[p1, p2]);

        assert_eq!(
            state.players.get(&p1).unwrap().mana_pool.black,
            0,
            "without spell mastery, no bonus {{B}}{{B}}{{B}} is added (pool was fully \
             spent paying the cast)"
        );
    }
}

// ── 6. Land Tax — upkeep search gated by opponent land count ───────────────────

#[test]
/// Oracle: "At the beginning of your upkeep, if an opponent controls more lands
/// than you, you may search your library for up to three basic land cards,
/// reveal them, put them into your hand, then shuffle." Verifies the search
/// fires (finding up to 3 basics) when an opponent controls more lands, and does
/// NOT fire when lands are equal.
fn test_land_tax_upkeep_search_gated_by_opponent_lands() {
    let def = mtg_engine::cards::defs::land_tax::card();

    // -- Opponent controls more lands: search finds up to 3 basics. --
    {
        let registry = CardRegistry::new(vec![def.clone()]);
        let p1 = p(1);
        let p2 = p(2);
        let mut b = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(
                ObjectSpec::card(p1, "Land Tax")
                    .with_card_id(CardId("land-tax".to_string()))
                    .with_types(vec![CardType::Enchantment])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(ObjectSpec::land(p1, "P1 Land"))
            .object(ObjectSpec::land(p2, "P2 Land A"))
            .object(ObjectSpec::land(p2, "P2 Land B"));
        for i in 0..4 {
            b = b.object(
                ObjectSpec::land(p1, &format!("Basic {i}"))
                    .with_supertypes(vec![SuperType::Basic])
                    .in_zone(ZoneId::Library(p1)),
            );
        }
        let mut state = b.active_player(p1).at_step(Step::Untap).build().unwrap();
        // CR 502: the untap step grants no priority, so the builder leaves the holder
        // unset. Seed it so we can pass out of untap (see tests/cumulative_upkeep.rs).
        state.turn.priority_holder = Some(p1);

        // CR 603.3: the upkeep trigger is queued on the stack at step entry; resolve it.
        let state = advance_to_step(state, Step::Upkeep);
        let state = resolve_stack(state, &[p1, p2]);

        let basics_in_hand = state
            .objects
            .iter()
            .filter(|(_, obj)| {
                obj.zone == ZoneId::Hand(p1) && obj.characteristics.name.starts_with("Basic")
            })
            .count();
        assert_eq!(
            basics_in_hand, 3,
            "an opponent controlling more lands must trigger a search for up to three \
             basic land cards into hand"
        );
    }

    // -- Lands are equal: search does NOT fire. --
    {
        let registry = CardRegistry::new(vec![def]);
        let p1 = p(1);
        let p2 = p(2);
        let mut b = GameStateBuilder::new()
            .add_player(p1)
            .add_player(p2)
            .with_registry(registry)
            .object(
                ObjectSpec::card(p1, "Land Tax")
                    .with_card_id(CardId("land-tax".to_string()))
                    .with_types(vec![CardType::Enchantment])
                    .in_zone(ZoneId::Battlefield),
            )
            .object(ObjectSpec::land(p1, "P1 Land Eq"))
            .object(ObjectSpec::land(p2, "P2 Land Eq"));
        b = b.object(
            ObjectSpec::land(p1, "Basic Unused")
                .with_supertypes(vec![SuperType::Basic])
                .in_zone(ZoneId::Library(p1)),
        );
        let mut state = b.active_player(p1).at_step(Step::Untap).build().unwrap();
        state.turn.priority_holder = Some(p1);

        // Resolve the upkeep stack, then stop. Passing further would reach the draw
        // step and pull "Basic Unused" into hand for an unrelated reason, which would
        // make the assertion below meaningless.
        //
        // NOTE (OOS-AC6-2): strictly, CR 603.4 says an intervening-if ability does not
        // trigger *at all* when its condition is false, so the stack should stay empty
        // here. The engine's generic upkeep sweep instead queues the trigger
        // unconditionally and evaluates `intervening_if` at resolution. That is
        // pre-existing sweep behavior shared by every `AtBeginningOfYourUpkeep` CardDef
        // trigger, not something PB-AC6 introduced, so this test asserts the observable
        // game state (no search occurred) rather than the stack contents.
        let state = advance_to_step(state, Step::Upkeep);
        let state = resolve_stack(state, &[p1, p2]);

        assert!(
            find_in_zone(&state, "Basic Unused", ZoneId::Hand(p1)).is_none(),
            "equal land counts must NOT perform the search (CR 603.4 intervening-if)"
        );
    }
}
