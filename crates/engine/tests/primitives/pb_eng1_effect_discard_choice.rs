//! ENG-1 (`scutemob-191`): effect-driven discard becomes a real CR 701.9b
//! player choice.
//!
//! CR 701.9a: "To discard a card, move it from its owner's hand to that
//! player's graveyard."
//! CR 701.9b: "By default, effects that cause a player to discard a card
//! allow the affected player to choose which card to discard. Some effects,
//! however, require a random discard or allow another player to choose which
//! card is discarded."
//! CR 608.2d: "If an effect of a spell or ability offers any choices other
//! than choices already made as part of casting the spell, activating the
//! ability, or otherwise putting the spell or ability on the stack, the
//! player announces these while applying the effect."
//! CR 702.35a (Madness): a discarded card with madness is exiled instead of
//! going to the graveyard; its owner may cast it for the madness cost.
//!
//! Before this batch `Effect::DiscardCards` called `discard_cards` straight
//! through, which took the LOWEST `ObjectId` in the affected player's hand --
//! never asking. See `memory/primitives/pb-plan-ENG1.md` for the full design;
//! this file exercises its §8 test list (a)-(h) plus the §13 risk 4 (both
//! loop exits) and risk 5 (nesting) mitigations.

use mtg_engine::cards::card_definition::{PlayerTarget, WheelDisposal, WheelDraw};
use mtg_engine::effects::{execute_effect, EffectContext};
use mtg_engine::rules::command::CastSpellData;
use mtg_engine::state::stack::StackObjectKind;
use mtg_engine::state::stubs::PendingTriggerKind;
use mtg_engine::testing::script_schema::EffectChoiceScriptAnswer;
use mtg_engine::{
    enrich_spec_from_def, process_command, AbilityDefinition, CardDefinition, CardEffectTarget,
    CardId, CardRegistry, CardType, Command, Condition, Cost, Effect, EffectAmount,
    EffectChoiceAnswer, EffectChoiceQuestion, GameEvent, GameState, GameStateBuilder,
    KeywordAbility, ManaColor, ManaCost, ObjectId, ObjectSpec, PlayerId, Step, TargetRequirement,
    TypeLine, ZoneId,
};

// ── Shared helpers (pattern: `pb_dp9_effect_choice.rs` / `pb_dp7_cleanup_discard.rs`) ──

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_obj(state: &GameState, name: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("object '{}' not found in state", name))
}

fn zone_of(state: &GameState, name: &str) -> Option<ZoneId> {
    state
        .objects()
        .values()
        .find(|o| o.characteristics.name == name)
        .map(|o| o.zone)
}

fn name_of(state: &GameState, id: ObjectId) -> String {
    state
        .objects()
        .get(&id)
        .map(|o| o.characteristics.name.clone())
        .unwrap_or_else(|| "<gone>".to_string())
}

fn hand_ids(state: &GameState, player: PlayerId) -> Vec<ObjectId> {
    let mut ids = state
        .zone(&ZoneId::Hand(player))
        .map(|z| z.object_ids())
        .unwrap_or_default();
    ids.sort();
    ids
}

fn defs_of(def: &CardDefinition) -> std::collections::HashMap<String, CardDefinition> {
    let mut m = std::collections::HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

/// Pass priority once per listed player. No pump -- every test in this file
/// wants to observe the suspension, not skip past it.
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

/// Answer the outstanding choice with an explicitly supplied answer.
fn answer_with(state: GameState, answer: EffectChoiceAnswer) -> (GameState, Vec<GameEvent>) {
    let entry = state
        .pending_effect_choice()
        .expect("no CR 608.2d resolution-time choice is pending");
    let player = entry.player;
    let choice_id = entry.choice_id;
    process_command(
        state,
        Command::AnswerEffectChoice {
            player,
            choice_id,
            answer,
        },
    )
    .expect("answer should be accepted")
}

fn cast(player: PlayerId, card: ObjectId) -> Command {
    Command::CastSpell(Box::new(CastSpellData {
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
    }))
}

fn spell_def(name: &str, card_id: &str, effect: Effect) -> CardDefinition {
    CardDefinition {
        name: name.to_string(),
        card_id: CardId(card_id.to_string()),
        mana_cost: Some(ManaCost {
            generic: 1,
            ..ManaCost::default()
        }),
        types: TypeLine {
            card_types: imbl::ordset![CardType::Sorcery],
            ..Default::default()
        },
        abilities: vec![AbilityDefinition::Spell {
            effect,
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// "Discard Spell" -- `Effect::DiscardCards { player: <target>, count: n }`.
fn discard_spell(target: PlayerTarget, n: i32) -> CardDefinition {
    spell_def(
        "Discard Spell",
        "eng1-discard-spell",
        Effect::DiscardCards {
            player: target,
            count: EffectAmount::Fixed(n),
        },
    )
}

/// "Group Discard Spell" -- `Effect::DiscardCards { player: EachOpponent, .. }`,
/// used only by the §13 risk 4 multiplayer test.
fn each_opponent_discard_spell(n: i32) -> CardDefinition {
    spell_def(
        "Group Discard Spell",
        "eng1-group-discard-spell",
        Effect::DiscardCards {
            player: PlayerTarget::EachOpponent,
            count: EffectAmount::Fixed(n),
        },
    )
}

/// "Wheel Hand Spell" -- `Effect::WheelHand { disposal: Discard, draw }`.
fn wheel_hand_spell(draw: WheelDraw) -> CardDefinition {
    spell_def(
        "Wheel Hand Spell",
        "eng1-wheel-hand-spell",
        Effect::WheelHand {
            player: PlayerTarget::Controller,
            disposal: WheelDisposal::Discard,
            draw,
        },
    )
}

fn hand_card(owner: PlayerId, name: &str) -> ObjectSpec {
    ObjectSpec::card(owner, name)
        .in_zone(ZoneId::Hand(owner))
        .with_types(vec![CardType::Instant])
}

/// Fiery Temper: Instant {1}{R}{R}, "... Madness {R}" -- the same minimal def
/// `pb_dp7_cleanup_discard.rs` and `mechanics_m_z/madness.rs` use.
fn fiery_temper_def() -> CardDefinition {
    CardDefinition {
        card_id: CardId("fiery-temper".to_string()),
        name: "Fiery Temper".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 2,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Fiery Temper deals 3 damage to any target. Madness {R}".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Madness),
            AbilityDefinition::Madness {
                cost: ManaCost {
                    red: 1,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    source: None,
                    target: CardEffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                targets: vec![TargetRequirement::TargetPlayerOrPlaneswalker],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}

/// A two-player fixture whose only spell is `def`, in p1's hand, plus
/// whatever `extra` objects the caller wants. p1 has 5 colourless floating.
fn fixture(def: CardDefinition, extra: Vec<ObjectSpec>) -> GameState {
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![def.clone()]))
        .active_player(p(1))
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p(1), &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p(1))),
        );
    for spec in extra {
        builder = builder.object(spec);
    }
    let mut state = builder.build().unwrap();
    state
        .players_mut()
        .get_mut(&p(1))
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p(1));
    state
}

/// Cast p1's only spell and pass both players -- leaving the game at
/// whatever the resolution produced (suspended, or complete).
fn cast_and_resolve(state: GameState, spell_name: &str) -> (GameState, Vec<GameEvent>) {
    let spell_id = state
        .objects()
        .iter()
        .find(|(_, o)| o.characteristics.name == spell_name && o.zone == ZoneId::Hand(p(1)))
        .map(|(id, _)| *id)
        .expect("spell should be in p1's hand");
    let (state, _) = process_command(state, cast(p(1), spell_id)).expect("cast should succeed");
    pass_all(state, &[p(1), p(2)])
}

#[allow(clippy::too_many_arguments)]
fn translate_answer_effect_choice(
    player: PlayerId,
    effect_choice: &EffectChoiceScriptAnswer,
    state: &GameState,
) -> Option<Command> {
    mtg_engine::translate_player_action(
        "answer_effect_choice",
        player,
        None, // card_name
        0,    // ability_index
        &[],  // targets
        &[],  // attackers_decl
        &[],  // blockers_decl
        &[],  // convoke_names
        &[],  // improvise_names
        &[],  // delve_names
        &[],  // escape_names
        false,
        false,
        &[],    // enlist_decls
        None,   // attacker_name
        None,   // discard_land_name
        None,   // discard_card_name
        None,   // bargain_sacrifice_name
        None,   // emerge_sacrifice_name
        None,   // casualty_sacrifice_name
        None,   // assist_player_name
        0,      // assist_amount
        0,      // replicate_count
        &[],    // splice_card_names
        0,      // escalate_modes
        vec![], // modes_chosen
        None,   // target_creature_name
        0,      // x_value
        &[],    // collect_evidence_names
        0,      // squad_count
        false,  // mutate_on_top
        None,   // gift_opponent_name
        None,   // sacrifice_card_name
        &[],    // exert_names
        None,   // pitch_exile_card_name
        None,   // chosen_color_name
        &[],    // hybrid_choice_names
        &[],    // phyrexian_life_payment_choices
        &[],    // discard_cards (PB-DP7 cleanup channel) -- not used here
        &[],    // trigger_targets (PB-DP8) -- not used here
        Some(effect_choice),
        state,
        &std::collections::HashMap::new(),
    )
}

// ── (a) — the suspension names the AFFECTED player, not the controller ─────

/// CR 701.9b / CR 608.2d: a real `Complete` def (`fell_specter`) whose ETB
/// makes a TARGETED OPPONENT discard. The resolution suspends, and the
/// pending choice's `player` is that OPPONENT -- never `ctx.controller` (P1,
/// the caster). This is the point CR 701.9b actually makes: "the AFFECTED
/// player" chooses, and here that is whoever the ETB targeted.
#[test]
fn test_eng1_effect_discard_suspends_for_the_affected_player() {
    let p1 = p(1);
    let p2 = p(2);
    let def = mtg_engine::cards::defs::fell_specter::card();
    let defs = defs_of(&def);
    let registry = CardRegistry::new(vec![def.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .object(enrich_spec_from_def(
            ObjectSpec::card(p1, "Fell Specter")
                .with_card_id(def.card_id.clone())
                .in_zone(ZoneId::Hand(p1)),
            &defs,
        ))
        .object(hand_card(p2, "P2 Alpha"))
        .object(hand_card(p2, "P2 Beta"))
        .object(hand_card(p2, "P2 Gamma"))
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .build()
        .unwrap();
    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 3);
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Black, 1);
    state.turn_mut().priority_holder = Some(p1);

    let spell_id = find_obj(&state, "Fell Specter");
    let (state, _) = process_command(state, cast(p1, spell_id)).expect("cast must succeed");
    let (state, _) = pass_all(state, &[p1, p2]); // resolve creature spell -> ETB queued
    let (state, _) = pass_all(state, &[p1, p2]); // resolve ETB trigger -> DiscardCards suspends

    let entry = state
        .pending_effect_choice()
        .expect("CR 701.9b: the discard must ask");
    assert_eq!(
        entry.player, p2,
        "CR 701.9b: the AFFECTED player is the TARGETED OPPONENT, not ctx.controller (P1)"
    );
    assert_ne!(
        entry.player, p1,
        "the controller must NEVER be asked for an opponent's discard"
    );

    let mut expected_hand = hand_ids(&state, p2);
    expected_hand.sort();
    assert_eq!(
        entry.question,
        EffectChoiceQuestion::Discard {
            hand: expected_hand,
            count: 1
        },
        "CR 608.2d: the question carries the full legal answer space -- P2's whole hand"
    );
}

/// §13 risk 4: the `for p in players` loop's TWO exits, both executed in one
/// resolution. `Effect::DiscardCards { player: EachOpponent, .. }` iterates
/// players ascending: P2's single-card hand is DETERMINED (`n == hand.len()`)
/// and takes the `continue` exit; P3's larger hand asks and takes the
/// `None => return` exit. Getting these backwards is not a compile error.
#[test]
fn test_eng1_multiplayer_discard_exercises_both_loop_exits() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);
    let def = each_opponent_discard_spell(1);
    let registry = CardRegistry::new(vec![def.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, &def.name)
                .with_card_id(def.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        .object(hand_card(p2, "P2 Only Card"))
        .object(hand_card(p3, "P3 Card A"))
        .object(hand_card(p3, "P3 Card B"))
        .object(hand_card(p3, "P3 Card C"))
        .build()
        .unwrap();
    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p1);

    let spell_id = find_obj(&state, "Group Discard Spell");
    let (state, _) = process_command(state, cast(p1, spell_id)).expect("cast should succeed");
    let (state, _) = pass_all(state, &[p1, p2, p3]);

    // CONTINUE exit: P2's determined discard has NOT actually happened yet --
    // the suspension rolled back the WHOLE resolution, including whatever P2
    // segment already ran before P3 was reached.
    assert_eq!(
        hand_ids(&state, p2).len(),
        1,
        "P2's card must still be in hand -- the suspension rolled EVERYTHING back"
    );
    let entry = state
        .pending_effect_choice()
        .expect("P3's larger hand must ask");
    assert_eq!(
        entry.player, p3,
        "P2 was determined (no ask, continue); P3 is the one who actually asks"
    );

    let p3_hand = hand_ids(&state, p3);
    let chosen = p3_hand[1]; // the MIDDLE id -- neither the default (lowest) nor an edge index
    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::Discard {
            chosen: vec![chosen],
        },
    );

    // Now BOTH exits have actually applied on the replay: P2's single card is
    // gone (the CONTINUE exit, applied for real this time) and P3's CHOSEN
    // card is gone (the RETURN exit's suspension, now answered).
    assert_eq!(
        hand_ids(&state, p2).len(),
        0,
        "P2's determined discard must have applied on the replay"
    );
    assert_eq!(
        hand_ids(&state, p3).len(),
        2,
        "exactly P3's chosen card must be gone"
    );
    assert!(
        !hand_ids(&state, p3).contains(&chosen),
        "P3's specifically CHOSEN card must be the one discarded"
    );
}

// ── (b) — the test that would have caught the shipped defect ───────────────

/// CR 701.9b: answering with the HIGHEST id (never the pre-batch auto-pick)
/// discards THAT card, and the auto-pick (`hand[0]`, the lowest id) remains
/// untouched in hand.
#[test]
fn test_eng1_a_non_default_answer_discards_the_chosen_card() {
    let state = fixture(
        discard_spell(PlayerTarget::Controller, 1),
        vec![
            hand_card(p(1), "Alpha"),
            hand_card(p(1), "Beta"),
            hand_card(p(1), "Gamma"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Discard Spell");
    let entry = state.pending_effect_choice().unwrap().clone();
    let hand = match &entry.question {
        EffectChoiceQuestion::Discard { hand, .. } => hand.clone(),
        other => panic!("expected a discard question, got {other:?}"),
    };
    let (lowest, highest) = (hand[0], *hand.last().unwrap());
    assert_ne!(lowest, highest, "the fixture must offer a real choice");
    let lowest_name = name_of(&state, lowest);
    let highest_name = name_of(&state, highest);

    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::Discard {
            chosen: vec![highest],
        },
    );

    assert!(
        matches!(zone_of(&state, &highest_name), Some(ZoneId::Graveyard(_))),
        "the CHOSEN (highest-id) card must be discarded"
    );
    assert!(
        matches!(zone_of(&state, &lowest_name), Some(ZoneId::Hand(_))),
        "the pre-ENG-1 auto-pick (lowest id) must remain in hand -- this is the \
         test that would have caught the shipped defect"
    );
}

// ── (c) — four determined shapes, one test ──────────────────────────────────

/// CR 601.2c's principle / CR 608.2d: when the answer space admits exactly
/// ONE legal answer, nothing is announced. `count > hand.len()`,
/// `count == hand.len()`, an empty hand, and `count == 0` against a
/// non-empty hand.
#[test]
fn test_eng1_a_determined_discard_does_not_suspend() {
    // count > hand.len(): 5 requested, 3 available.
    {
        let state = fixture(
            discard_spell(PlayerTarget::Controller, 5),
            vec![
                hand_card(p(1), "A"),
                hand_card(p(1), "B"),
                hand_card(p(1), "C"),
            ],
        );
        let (state, _) = cast_and_resolve(state, "Discard Spell");
        assert!(
            state.pending_effect_choice().is_none(),
            "count > hand.len() must not suspend"
        );
        assert_eq!(
            hand_ids(&state, p(1)).len(),
            0,
            "the whole (smaller) hand is discarded"
        );
    }
    // count == hand.len(): the whole hand, exactly.
    {
        let state = fixture(
            discard_spell(PlayerTarget::Controller, 3),
            vec![
                hand_card(p(1), "A"),
                hand_card(p(1), "B"),
                hand_card(p(1), "C"),
            ],
        );
        let (state, _) = cast_and_resolve(state, "Discard Spell");
        assert!(
            state.pending_effect_choice().is_none(),
            "count == hand.len() must not suspend"
        );
        assert_eq!(hand_ids(&state, p(1)).len(), 0);
    }
    // empty hand.
    {
        let state = fixture(discard_spell(PlayerTarget::Controller, 1), vec![]);
        let (state, _) = cast_and_resolve(state, "Discard Spell");
        assert!(
            state.pending_effect_choice().is_none(),
            "an empty hand must not suspend"
        );
        assert_eq!(hand_ids(&state, p(1)).len(), 0);
    }
    // count == 0 against a non-empty hand -- the OTHER determined case;
    // nothing moves.
    {
        let state = fixture(
            discard_spell(PlayerTarget::Controller, 0),
            vec![hand_card(p(1), "A"), hand_card(p(1), "B")],
        );
        let (state, _) = cast_and_resolve(state, "Discard Spell");
        assert!(
            state.pending_effect_choice().is_none(),
            "count == 0 must not suspend"
        );
        assert_eq!(
            hand_ids(&state, p(1)).len(),
            2,
            "count == 0 must discard nothing"
        );
    }
}

// ── (d) — WheelHand: the short-circuit does NOT protect it; STRUCTURE does ──

/// §3.3: `Effect::WheelHand` never reaches the `Effect::DiscardCards` arm at
/// all -- it calls `discard_cards` DIRECTLY. This is a STRUCTURAL guarantee:
/// a 4-card hand discards exactly 4 (never 8, the double-count a suspend/
/// replay would cause), 4 are drawn, and `pending_effect_choice()` is `None`
/// throughout. Covers both `WheelDraw::ThatMany` and
/// `WheelDraw::GreatestDiscarded`.
#[test]
fn test_eng1_wheel_hand_discards_the_whole_hand_exactly_once_and_never_suspends() {
    for draw in [WheelDraw::ThatMany, WheelDraw::GreatestDiscarded] {
        let def = wheel_hand_spell(draw.clone());
        // CR 111.7: a token in any zone but the battlefield ceases to exist
        // as an SBA -- a token-based library filler would vanish the instant
        // it was drawn into hand. Real (non-token) card objects, same as the
        // hand cards, survive the draw.
        let state = fixture(
            def,
            vec![
                hand_card(p(1), "W1"),
                hand_card(p(1), "W2"),
                hand_card(p(1), "W3"),
                hand_card(p(1), "W4"),
                ObjectSpec::card(p(1), "Lib 0").in_zone(ZoneId::Library(p(1))),
                ObjectSpec::card(p(1), "Lib 1").in_zone(ZoneId::Library(p(1))),
                ObjectSpec::card(p(1), "Lib 2").in_zone(ZoneId::Library(p(1))),
                ObjectSpec::card(p(1), "Lib 3").in_zone(ZoneId::Library(p(1))),
            ],
        );
        assert!(
            state.pending_effect_choice().is_none(),
            "sanity: nothing pending before cast"
        );

        let (state, events) = cast_and_resolve(state, "Wheel Hand Spell");

        assert!(
            state.pending_effect_choice().is_none(),
            "Effect::WheelHand must NEVER suspend -- it calls discard_cards \
             directly, bypassing the Effect::DiscardCards arm entirely (§2.4/§3.3)"
        );
        assert!(
            !events
                .iter()
                .any(|e| matches!(e, GameEvent::EffectChoiceRequired { .. })),
            "no question event may be emitted for a WheelHand discard; draw={:?}",
            draw
        );
        assert_eq!(
            state.zone(&ZoneId::Graveyard(p(1))).unwrap().len(),
            5,
            "exactly 4 discarded hand cards + the spell itself (which also \
             resolves to the graveyard) -- NOT 9, which a suspend/replay \
             double-count of the 4 hand cards would produce (draw {:?})",
            draw
        );
        assert_eq!(
            hand_ids(&state, p(1)).len(),
            4,
            "4 cards drawn after disposal (draw {:?})",
            draw
        );
    }
}

// ── review Finding 6 (LOW) — the `Cost::DiscardCard` structural guarantee ───
// has its own regression guard, not just (d)'s WheelHand guard

/// CR 701.9c / §2.4 / `OOS-ENG1-1`: `Cost::DiscardCard` pays through
/// `discard_cards` directly, on a cost-payment path inside
/// `pay_optional_cost` with NO resolution wrapper to roll back to (unlike
/// `Effect::DiscardCards`, which is asked and can suspend/replay). An ask on
/// this path would record a `pending_effect_choice` nothing can discharge --
/// the trap-state class `OOS-DP9-14` was filed for.
///
/// (d) guards the SAME structural property for `Effect::WheelHand`, but only
/// that one: `WheelHand`'s call sites pass `n == hand_size`, so a future
/// batch that moved BOTH the ask and the short-circuit into `discard_cards`
/// would leave (d) green (the short-circuit would still fire for a
/// whole-hand discard) while a `Cost::DiscardCard` payment -- `n = 1` against
/// a LARGER hand, where the short-circuit does not apply -- would start
/// suspending. This test pins that case directly.
#[test]
fn test_eng1_a_cost_discard_never_suspends() {
    let p1 = p(1);
    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p(2))
        .with_registry(CardRegistry::new(vec![]))
        .object(ObjectSpec::card(p1, "Hand A").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Hand B").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Hand C").in_zone(ZoneId::Hand(p1)))
        .object(ObjectSpec::card(p1, "Lib Card").in_zone(ZoneId::Library(p1)))
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();
    assert!(
        state.pending_effect_choice().is_none(),
        "sanity: nothing pending before the cost is paid"
    );

    let effect = Effect::MayPayThenEffect {
        cost: Cost::DiscardCard,
        payer: PlayerTarget::Controller,
        then: Box::new(Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        }),
    };
    let mut ctx = EffectContext::new(p1, ObjectId(0), vec![]);
    let mut state = state;
    let events = execute_effect(&mut state, &effect, &mut ctx);

    // The cost path must have actually run -- a hand of 3 (> 1) means the
    // cost is payable, so this is not a "nothing happened" pass.
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { player, .. } if *player == p1)),
        "sanity: the cost must actually have been paid for this test to prove \
         anything; events={events:?}"
    );
    assert!(
        state.pending_effect_choice().is_none(),
        "CR 701.9c / §2.4: a Cost::DiscardCard payment must NEVER suspend -- it \
         calls discard_cards directly, bypassing the Effect::DiscardCards arm's \
         ask (structural guarantee, OOS-ENG1-1). A larger hand than the amount \
         discarded rules out the n >= hand.len() short-circuit as an alternate \
         explanation for a passing result."
    );
}

// ── (e) — the two defaults are deliberately opposite ends of the same hand ──

/// CR 608.2d/701.9b vs CR 514.1: `default_discard_answer` and
/// `default_cleanup_discard` are the OPPOSITE ends of the sorted hand,
/// pinned side by side deliberately -- they reproduce two auto-picks that
/// genuinely differed (`min_by_key` vs `obj_ids().last()`).
#[test]
fn test_eng1_defaults_reproduce_both_pre_batch_picks() {
    // The effect-driven half is a pure function of the question -- no
    // GameState required.
    let hand: Vec<ObjectId> = (100..105).map(ObjectId).collect();
    let question = EffectChoiceQuestion::Discard {
        hand: hand.clone(),
        count: 2,
    };
    let answer = mtg_engine::effects::default_discard_answer(&question);
    assert_eq!(
        answer,
        EffectChoiceAnswer::Discard {
            chosen: hand[0..2].to_vec()
        },
        "CR 701.9b: default_discard_answer takes the LOWEST ids"
    );

    // The cleanup half needs a real GameState with a real
    // `pending_cleanup_discard` entry: CR 402.2's default max hand size 7
    // makes a 9-card hand produce a count of 2 through the real cleanup
    // flow, so this is 9 cards rather than the plan's illustrative 5 -- the
    // point (the SAME hand answering the SAME count from opposite ends) is
    // unaffected by the exact size.
    let mut builder = GameStateBuilder::new()
        .add_player(p(1))
        .add_player(p(2))
        .active_player(p(1))
        .at_step(Step::End);
    for i in 0..9u32 {
        builder = builder
            .object(ObjectSpec::card(p(1), &format!("Card {i}")).in_zone(ZoneId::Hand(p(1))));
    }
    let state = builder.build().unwrap();
    let (state, _) = pass_all(state, &[p(1), p(2)]);
    let entry = state
        .pending_cleanup_discard()
        .expect("a 9-card hand must trigger CR 514.1");
    assert_eq!(entry.count, 2);
    let nine_ids = hand_ids(&state, p(1));
    let expected_highest = nine_ids[nine_ids.len() - 2..].to_vec();

    let default = mtg_engine::rules::turn_actions::default_cleanup_discard(&state, p(1));
    assert_eq!(
        default, expected_highest,
        "CR 514.1: default_cleanup_discard takes the HIGHEST ids -- the OPPOSITE \
         end from default_discard_answer above, deliberately"
    );
}

// ── (f) — five rejections, each a distinct message ──────────────────────────

/// CR 701.9b / CR 608.2d: five rejections, each a distinct message, each
/// leaving the block intact and the hand unchanged. (1) wrong variant
/// (check 4); (2) an id not in hand (check 5); (3) wrong count; (4) a
/// duplicate id; (5) the SR-29 half -- a DIFFERENT seat, refused by check 2.
#[test]
fn test_eng1_illegal_discard_answers_are_refused_and_leave_the_state_untouched() {
    let state = fixture(
        discard_spell(PlayerTarget::Controller, 2),
        vec![
            hand_card(p(1), "Alpha"),
            hand_card(p(1), "Beta"),
            hand_card(p(1), "Gamma"),
            hand_card(p(1), "Delta"),
        ],
    );
    let (state, _) = cast_and_resolve(state, "Discard Spell");
    let entry = state.pending_effect_choice().unwrap().clone();
    let hand = match &entry.question {
        EffectChoiceQuestion::Discard { hand, .. } => hand.clone(),
        other => panic!("expected a discard question, got {other:?}"),
    };
    let hash = state.public_state_hash();

    let cases: Vec<(&str, PlayerId, u64, EffectChoiceAnswer, &str)> = vec![
        (
            "wrong variant (check 4)",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::Scry {
                bottom: vec![],
                top: vec![],
            },
            "does not answer question",
        ),
        (
            "an id not in hand (check 5)",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::Discard {
                chosen: vec![hand[0], ObjectId(999_999)],
            },
            "not in the hand this effect is discarding from",
        ),
        (
            "wrong count",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::Discard {
                chosen: vec![hand[0]],
            },
            "discards exactly 2",
        ),
        (
            "a duplicate id",
            entry.player,
            entry.choice_id,
            EffectChoiceAnswer::Discard {
                chosen: vec![hand[0], hand[0]],
            },
            "named more than once",
        ),
        (
            "a DIFFERENT seat answers (SR-29)",
            p(2),
            entry.choice_id,
            EffectChoiceAnswer::Discard {
                chosen: vec![hand[0], hand[1]],
            },
            "608.2d",
        ),
    ];
    for (label, player, choice_id, answer, expect) in cases {
        let mut probe = state.clone();
        let err =
            mtg_engine::effects::handle_answer_effect_choice(&mut probe, player, choice_id, answer)
                .expect_err(label);
        let msg = format!("{err:?}");
        assert!(
            msg.contains(expect),
            "{label}: expected a message naming {expect:?}, got {msg}"
        );
        assert_eq!(
            probe.public_state_hash(),
            hash,
            "{label}: a rejected answer must leave the state untouched"
        );
        assert_eq!(
            hand_ids(&probe, p(1)).len(),
            4,
            "{label}: the hand must be unchanged"
        );
    }

    // Control: an ACCEPTED answer DOES move the hash, so the rejections above
    // cannot be passing for the wrong reason.
    let mut accepted = state.clone();
    mtg_engine::effects::handle_answer_effect_choice(
        &mut accepted,
        entry.player,
        entry.choice_id,
        EffectChoiceAnswer::Discard {
            chosen: vec![hand[0], hand[1]],
        },
    )
    .expect("a legal answer must be accepted");
    assert_ne!(
        accepted.public_state_hash(),
        hash,
        "an accepted answer must change the state"
    );
}

// ── (g) — an explicitly chosen Madness card still routes to exile ──────────

/// CR 702.35a / CR 701.9b: an EXPLICITLY chosen Madness card routes through
/// the SAME `discard_one_chosen_card` body the auto-pick uses -- exile, not
/// graveyard; `CardDiscarded` still fires; a `PendingTriggerKind::Madness`
/// is queued with the printed cost. A copy-pasted second implementation
/// would pass every other test in this file and fail this one.
#[test]
fn test_eng1_a_chosen_madness_card_still_routes_to_exile() {
    let p1 = p(1);
    let p2 = p(2);
    let spell = discard_spell(PlayerTarget::Controller, 1);
    let temper = fiery_temper_def();
    let registry = CardRegistry::new(vec![spell.clone(), temper.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(registry)
        .active_player(p1)
        .at_step(Step::PreCombatMain)
        .object(
            ObjectSpec::card(p1, &spell.name)
                .with_card_id(spell.card_id.clone())
                .with_types(vec![CardType::Sorcery])
                .with_mana_cost(ManaCost {
                    generic: 1,
                    ..ManaCost::default()
                })
                .in_zone(ZoneId::Hand(p1)),
        )
        // Filler A/B are added FIRST -- their ids are LOWER than Fiery
        // Temper's, so the DEFAULT (lowest-id) pick would choose a filler,
        // never Temper. The explicit choice below must be the reason Temper
        // is discarded, per `enrich_spec_from_def()`'s warning: a naked spec
        // has no Madness keyword unless it is attached explicitly (mirroring
        // `pb_dp7_cleanup_discard.rs::build_oversized_hand`'s proven recipe).
        .object(hand_card(p1, "Filler A"))
        .object(hand_card(p1, "Filler B"))
        .object(
            ObjectSpec::card(p1, "Fiery Temper")
                .with_card_id(temper.card_id.clone())
                .with_keyword(KeywordAbility::Madness)
                .in_zone(ZoneId::Hand(p1)),
        )
        .build()
        .unwrap();
    let mut state = state;
    state
        .players_mut()
        .get_mut(&p1)
        .unwrap()
        .mana_pool
        .add(ManaColor::Colorless, 5);
    state.turn_mut().priority_holder = Some(p1);

    let spell_id = find_obj(&state, "Discard Spell");
    let (state, _) = process_command(state, cast(p1, spell_id)).unwrap();
    let (state, _) = pass_all(state, &[p1, p2]);

    let entry = state
        .pending_effect_choice()
        .expect("the 3-card hand must ask");
    let temper_id = find_obj(&state, "Fiery Temper");
    let default_pick = match &entry.question {
        EffectChoiceQuestion::Discard { hand, .. } => hand[0],
        other => panic!("expected a discard question, got {other:?}"),
    };
    assert_ne!(
        temper_id, default_pick,
        "the fixture must make Fiery Temper NOT the default (lowest-id) pick, or \
         this test cannot discriminate an explicit choice from the default"
    );

    let (state, events) = answer_with(
        state,
        EffectChoiceAnswer::Discard {
            chosen: vec![temper_id],
        },
    );

    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::CardDiscarded { .. })),
        "CR ruling: CardDiscarded still fires even though the card goes to exile"
    );
    assert!(
        state
            .objects()
            .values()
            .any(|o| o.characteristics.name == "Fiery Temper" && o.zone == ZoneId::Exile),
        "CR 702.35a: the chosen Madness card goes to EXILE, not the graveyard"
    );
    let madness_pending = state
        .pending_triggers()
        .iter()
        .filter(|t| t.kind == PendingTriggerKind::Madness)
        .count();
    let madness_on_stack = state
        .stack_objects()
        .iter()
        .filter(|so| matches!(so.kind, StackObjectKind::MadnessTrigger { .. }))
        .count();
    assert_eq!(
        madness_pending + madness_on_stack,
        1,
        "exactly one Madness trigger must be queued or on the stack -- this is \
         what proves the announced path shares discard_one_chosen_card's body"
    );
    if madness_on_stack == 1 {
        let cost = state.stack_objects().iter().find_map(|so| {
            if let StackObjectKind::MadnessTrigger { madness_cost, .. } = &so.kind {
                Some(madness_cost.clone())
            } else {
                None
            }
        });
        assert_eq!(
            cost,
            Some(ManaCost {
                red: 1,
                ..Default::default()
            })
        );
    }
}

// ── (h) — the script harness can drive a named discard ─────────────────────

/// CR 608.2d (§3.9a): the script harness answers a named discard through
/// `translate_player_action("answer_effect_choice", ...)`, producing a
/// `Command::AnswerEffectChoice` naming the discarded card's id -- without
/// touching the SR-9c golden-script partition (no golden script is added).
#[test]
fn test_eng1_the_script_harness_can_drive_a_named_discard() {
    let state = fixture(
        discard_spell(PlayerTarget::Controller, 1),
        vec![hand_card(p(1), "Mountain"), hand_card(p(1), "Forest")],
    );
    let (state, _) = cast_and_resolve(state, "Discard Spell");
    assert!(
        state.pending_effect_choice().is_some(),
        "the 2-card hand must ask"
    );
    let mountain_id = find_obj(&state, "Mountain");

    let spec = EffectChoiceScriptAnswer {
        discard: vec!["Mountain".to_string()],
        ..Default::default()
    };
    let cmd = translate_answer_effect_choice(p(1), &spec, &state)
        .expect("translate_player_action should answer the discard");

    match cmd {
        Command::AnswerEffectChoice { player, answer, .. } => {
            assert_eq!(player, p(1));
            match answer {
                EffectChoiceAnswer::Discard { chosen } => {
                    assert_eq!(
                        chosen,
                        vec![mountain_id],
                        "the NAMED card must be the one chosen"
                    );
                }
                other => panic!("expected a Discard answer, got {other:?}"),
            }
        }
        other => panic!("expected AnswerEffectChoice, got {other:?}"),
    }
}

// ── §13 risk 5 — a discard nested one level deep suspends and replays ──────

/// A discard nested inside `Conditional` inside `Sequence` suspends and
/// replays correctly. The three siblings (SearchLibrary/Scry/Surveil,
/// `pb_dp9_effect_choice.rs::test_dp9_choice_inside_conditional_and_sequence`)
/// already prove the inherited machinery covers THREE variants; do not
/// assume it covers a FOURTH just because it covered three.
#[test]
fn test_eng1_discard_nested_in_conditional_and_sequence_suspends_and_replays() {
    let inner = Effect::Sequence(vec![
        Effect::DiscardCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::Fixed(5),
        },
    ]);
    let def = spell_def(
        "Nested Discard Spell",
        "eng1-nested-discard",
        Effect::Sequence(vec![
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(1),
            },
            Effect::Conditional {
                condition: Condition::Always,
                if_true: Box::new(inner),
                if_false: Box::new(Effect::Nothing),
            },
            Effect::GainLife {
                player: PlayerTarget::Controller,
                amount: EffectAmount::Fixed(7),
            },
        ]),
    );
    let state = fixture(def, vec![hand_card(p(1), "Alpha"), hand_card(p(1), "Beta")]);
    let life_before = state.players()[&p(1)].life_total;

    let (state, _) = cast_and_resolve(state, "Nested Discard Spell");
    assert!(
        state.pending_effect_choice().is_some(),
        "the nested discard must ask"
    );
    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before,
        "the roll-back undoes the +1 the Sequence had already applied"
    );

    let entry = state.pending_effect_choice().unwrap().clone();
    let hand = match &entry.question {
        EffectChoiceQuestion::Discard { hand, .. } => hand.clone(),
        other => panic!("expected a discard question, got {other:?}"),
    };
    let highest = *hand.last().unwrap(); // NOT the default (lowest) pick
    let highest_name = name_of(&state, highest);

    let (state, _) = answer_with(
        state,
        EffectChoiceAnswer::Discard {
            chosen: vec![highest],
        },
    );

    assert_eq!(
        state.players()[&p(1)].life_total,
        life_before + 1 + 5 + 7,
        "every instruction runs EXACTLY once: +1 before the choice, +5 after it \
         inside the Conditional, +7 after the Conditional"
    );
    assert!(
        matches!(zone_of(&state, &highest_name), Some(ZoneId::Graveyard(_))),
        "the CHOSEN card must be discarded"
    );
}
