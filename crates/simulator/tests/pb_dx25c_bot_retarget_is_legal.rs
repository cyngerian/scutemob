//! PB-DX25c (`OOS-DX25b-3`, plan §5.3, AC 6304) — the bot path reaches
//! `rules::retarget::plan_target_change` and produces a legal redirect.
//!
//! S1 drives Misdirection's cast through `mtg_simulator::legal_actions` +
//! `mtg_simulator::targeting::plan_targets` (the SAME `StubProvider`/
//! `RandomBot` machinery a real fuzz game uses), never a hand-built
//! `Command::CastSpell`. The victim spell is cast directly (a real
//! `Command::CastSpell`, mirroring `crates/engine/tests/primitives/
//! pb_dx25c_retarget_legality.rs`'s fixtures) -- AC 6304's subject is
//! Misdirection's OWN cast reaching the bot layer, not the victim's.

use mtg_engine::rules::command::CastSpellData;
use mtg_engine::{
    process_command, AbilityDefinition, CardDefinition, CardId, CardRegistry, CardType, Command,
    Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ManaCost, ManaPool, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, Target, TargetRequirement, TypeLine, ZoneId,
};
use mtg_simulator::targeting::plan_targets;
use mtg_simulator::{
    check_invariants, Bot, LegalAction, LegalActionProvider, RandomBot, StubProvider, TargetPlan,
};
use std::sync::Arc;

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

fn find_stack_obj_on_stack(state: &GameState, name_substr: &str) -> ObjectId {
    state
        .objects()
        .iter()
        .find(|(_, obj)| {
            obj.zone == ZoneId::Stack && obj.characteristics.name.contains(name_substr)
        })
        .map(|(id, _)| *id)
        .unwrap_or_else(|| panic!("no ZoneId::Stack object containing '{}' found", name_substr))
}

fn cast(
    state: GameState,
    player: PlayerId,
    card: ObjectId,
    targets: Vec<Target>,
) -> (GameState, Vec<GameEvent>) {
    let mut state = state;
    state.turn_mut().priority_holder = Some(player);
    process_command(
        state,
        Command::CastSpell(Box::new(CastSpellData {
            player,
            card,
            targets,
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
        })),
    )
    .unwrap_or_else(|e| panic!("cast must succeed: {:?}", e))
}

/// Same shape as the engine-side fixture's `resolve_top_of_stack`: pass
/// priority as whoever currently holds it until the top of the stack
/// resolves.
fn resolve_top_of_stack(mut state: GameState) -> (GameState, Vec<GameEvent>) {
    let start_len = state.stack_objects().len();
    let mut all_events = Vec::new();
    for _ in 0..20 {
        let holder = state
            .turn()
            .priority_holder
            .unwrap_or_else(|| panic!("no priority holder to resolve the stack"));
        let (s, ev) = process_command(state, Command::PassPriority { player: holder })
            .unwrap_or_else(|e| panic!("PassPriority by {:?} failed: {:?}", holder, e));
        state = s;
        all_events.extend(ev);
        if state.stack_objects().len() < start_len {
            return (state, all_events);
        }
    }
    panic!(
        "stack did not resolve after 20 passes; events: {:?}",
        all_events
    );
}

fn life_loss_player_def(name: &str, card_id: &str) -> CardDefinition {
    CardDefinition {
        card_id: CardId(card_id.to_string()),
        name: name.to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: TypeLine {
            card_types: [CardType::Instant].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: format!("{name}: target opponent loses 3 life."),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::LoseLife {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                amount: EffectAmount::Fixed(3),
            },
            targets: vec![TargetRequirement::TargetOpponent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}

/// S1 (AC 6304) — the redirect is legal when the whole chain is driven the
/// way a bot drives it.
#[test]
fn s1_bot_driven_misdirection_cast_redirects_legally() {
    let p1 = p(1);
    let p2 = p(2);
    let p3 = p(3);

    let life_loss = life_loss_player_def("PB-DX25c S1 Life Loss", "pb-dx25c-s1-lifeloss");
    let misdirection = mtg_engine::cards::defs::misdirection::card();
    let registry: Arc<CardRegistry> =
        CardRegistry::new(vec![misdirection.clone(), life_loss.clone()]);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .add_player(p3)
        .with_registry(registry)
        .player_mana(
            p1,
            ManaPool {
                colorless: 3,
                blue: 2,
                ..Default::default()
            },
        )
        .player_mana(
            p2,
            ManaPool {
                red: 1,
                ..Default::default()
            },
        )
        .object(
            ObjectSpec::card(p1, "Misdirection")
                .in_zone(ZoneId::Hand(p1))
                .with_card_id(misdirection.card_id.clone())
                .with_types(vec![CardType::Instant])
                // ObjectSpec::card() is naked (gotchas-infra.md) -- the
                // simulator's StubProvider offer layer reads `obj.
                // characteristics.mana_cost` directly (unlike the engine's
                // own `handle_cast_spell`, which reads the registry def), so
                // this MUST be set explicitly for the cast to be offered.
                .with_mana_cost(misdirection.mana_cost.clone().unwrap()),
        )
        .object(
            ObjectSpec::card(p2, "PB-DX25c S1 Life Loss")
                .in_zone(ZoneId::Hand(p2))
                .with_card_id(life_loss.card_id.clone())
                .with_types(vec![CardType::Instant])
                .with_mana_cost(life_loss.mana_cost.clone().unwrap()),
        )
        .at_step(Step::PreCombatMain)
        .active_player(p1)
        .build()
        .unwrap();

    // p2 casts "target opponent loses 3 life" at p3 (a real cast; not this
    // probe's subject -- AC 6304 is about Misdirection's OWN bot-driven cast).
    let life_loss_hand_id = find_obj(&state, "PB-DX25c S1 Life Loss");
    let (state, _) = cast(state, p2, life_loss_hand_id, vec![Target::Player(p3)]);
    let life_loss_card_id = find_stack_obj_on_stack(&state, "S1 Life Loss");

    // Reach Misdirection's cast through the bot layer: StubProvider offers
    // it, plan_targets chooses its target, RandomBot builds the Command.
    let mut state = state;
    state.turn_mut().priority_holder = Some(p1);
    let misdirection_hand_id = find_obj(&state, "Misdirection");
    let action = StubProvider
        .legal_actions(&state, p1)
        .into_iter()
        .find(|a| matches!(a, LegalAction::CastSpell { card, .. } if *card == misdirection_hand_id))
        .unwrap_or_else(|| panic!("StubProvider must offer casting Misdirection"));

    // Non-vacuity anchor (PB-DX25's T6 lesson: a probe must not compare a
    // fixture to itself) -- plan_targets must announce a REAL target, not
    // nothing.
    let plan = plan_targets(&state, p1, &action);
    let TargetPlan::Announce(announced) = &plan else {
        panic!(
            "plan_targets must announce a target for Misdirection, got {:?}",
            plan
        );
    };
    assert_eq!(
        announced,
        &vec![Target::Object(life_loss_card_id)],
        "the bot layer must announce the victim spell (the only single-target \
         spell on the stack) as Misdirection's target"
    );

    let mut bot = RandomBot::new(1, "s1-bot".into());
    let cmd = bot.choose_action(&state, p1, std::slice::from_ref(&action));
    let Command::CastSpell(cast_data) = &cmd else {
        panic!("expected a CastSpell command from the bot, got {:?}", cmd);
    };
    assert_eq!(
        cast_data.targets,
        vec![Target::Object(life_loss_card_id)],
        "the bot-built Command::CastSpell must carry the same target plan_targets announced"
    );

    let (state, _) = process_command(state, cmd).unwrap_or_else(|e| {
        panic!(
            "the engine must accept the bot-built Misdirection cast: {:?}",
            e
        )
    });

    let p1_life_before = state.players().get(&p1).unwrap().life_total;
    let p2_life_before = state.players().get(&p2).unwrap().life_total;
    let p3_life_before = state.players().get(&p3).unwrap().life_total;
    let (state, resolve_events) = resolve_top_of_stack(state);
    let new_target = resolve_events
        .iter()
        .find_map(|e| match e {
            GameEvent::TargetsChanged { new_targets, .. } => Some(new_targets.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("Misdirection must redirect: {:?}", resolve_events));

    // `/review` Finding T7: the redirect must land on a NEW target, not the
    // original p3 -- V2 in the engine-side revert matrix (`retarget.rs`'s
    // `?`) showed a `TargetsChanged` event CAN fire with an unchanged target
    // set if the all-or-nothing abort is dropped; this is the bot-path
    // assertion that would catch that shape reaching a bot-driven cast.
    assert_ne!(
        new_target[0].target,
        Target::Player(p3),
        "the redirect must change the target away from p3 (the original), \
         not merely emit an event that leaves it unchanged"
    );
    let new_target_player = match &new_target[0].target {
        Target::Player(pid) => *pid,
        other => panic!("expected the redirect to land on a player, got {other:?}"),
    };

    // The post-resolution target must satisfy its own requirement, checked
    // against the OFFER LAYER's own answer -- `legal_targets_per_slot` --
    // not against a literal, so this assertion cannot drift from what the
    // engine itself considers legal.
    let candidates = mtg_engine::legal_targets_per_slot(
        &state,
        p2, // so.controller (the victim's own caster, CR 109.5)
        life_loss_card_id,
        &[TargetRequirement::TargetOpponent],
    );
    assert!(
        candidates[0].contains(&new_target[0].target),
        "the redirected target {:?} must be a MEMBER of legal_targets_per_slot's \
         own TargetOpponent answer {:?} -- if this fails, the retarget and the \
         offer layer disagree about what 'legal' means",
        new_target[0].target,
        candidates[0]
    );

    let (state, _) = resolve_top_of_stack(state);
    let violations = check_invariants(&state, None);
    assert!(
        violations.is_empty(),
        "the final state must carry zero invariant violations, got: {:?}",
        violations
    );

    // `/review` Finding T7: the life-total observables were computed
    // (`p3_life_before` et al.) and then discarded (`let _ = p3_life_before;`)
    // -- assert them for real. Whichever player the redirect landed on must
    // be the ONE who lost 3 life; p3 (the original target) and the player who
    // was NOT chosen must both be untouched.
    let p1_life_after = state.players().get(&p1).unwrap().life_total;
    let p2_life_after = state.players().get(&p2).unwrap().life_total;
    let p3_life_after = state.players().get(&p3).unwrap().life_total;
    assert_eq!(
        p3_life_after, p3_life_before,
        "p3 (the original, un-redirected-to target) must be untouched"
    );
    // p2 (so.controller, the victim's own caster) can never be a legal
    // TargetOpponent target for its own spell (CR 102.3 self-exclusion), so
    // p1 -- the only other player -- is the sole legal candidate. Asserted,
    // not assumed: the earlier membership check already proved it satisfies
    // legal_targets_per_slot; this pins WHICH player, so a wrong-player
    // redirect that still happens to satisfy membership cannot pass silently.
    assert_eq!(
        new_target_player, p1,
        "p2 (the victim's own caster) can never legally be its own \
         TargetOpponent, and p3 is excluded as the current target -- p1 is \
         the only legal candidate"
    );
    assert_eq!(
        p1_life_after,
        p1_life_before - 3,
        "p1 (the redirected target) must lose 3 life"
    );
    assert_eq!(
        p2_life_after, p2_life_before,
        "p2 (the victim's own caster, never a legal target for it) must be untouched"
    );
}
