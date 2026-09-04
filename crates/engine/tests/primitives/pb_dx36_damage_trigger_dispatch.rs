//! PB-DX36 (`OOS-CARDS2-6`) — behavioural probes for `DamageRecipient`,
//! `TriggerCondition::WhenDealsDamage`, the now-genuinely-read `combat_only` flag
//! on `WhenEnchantedCreatureDealsDamageToPlayer`, and `EffectAmount::DamageDealt`.
//!
//! Design record: `memory/primitives/pb-DX36-execution-notes.md` §0 (binding) and
//! `memory/primitives/pb-plan-DX36.md` step 8.
//!
//! Every "the trigger fired" assertion here counts, never `>= 1`
//! (execution notes §0.5(c) — `>= 1` is exactly the assertion shape PB-DX47's
//! double-push defect would still satisfy). `t2` is the dedicated exactly-once
//! probe on a COMBAT event, since the combat arm is the one that fires BOTH the
//! `…CombatDamage…` and `…AnyDamage…` `TriggerEvent`s per §0.5(c) — a
//! single-dispatch bug there is the one a `>= 1` count hides. **`t2` drives
//! `attack_unblocked`, a SINGLE-ASSIGNMENT fixture (one attacker, one
//! defending player, no blockers) — it proves the two-`TriggerEvent`-family
//! dispatch is exactly-once, and it proves NOTHING about a source with MORE
//! THAN ONE assignment in the same event (multi-block, trample), because a
//! single-assignment fixture cannot distinguish "dispatch once per event" from
//! "dispatch once per assignment" — PB-DX47's own lesson, that a differential
//! probe proves agreement on the branches it drives and nothing about the
//! branches it does not. `t8`/`t9` below are the multi-assignment probes
//! (`/review` HIGH 1) that `t2` cannot be.
//!
//! Two Aura defs are used: the real, deck-legal `Complete` `Sigil of Sleep`
//! (`combat_only: false`, `recipient: Player`) for the corpus-integration probes
//! (`t1`, `t2`), and small synthetic `CardDefinition`s built inline for the
//! `combat_only`/`recipient` combinations no corpus member declares today
//! (`combat_only: true` has zero declared corpus members — execution notes
//! §0.5(b); `recipient: Opponent` combined with a self-damage scenario has none
//! either). Both routes go through the REAL lowering
//! (`enrich_spec_from_def` → `build_face_triggered_abilities`), never a
//! hand-picked `TriggerEvent` — the point of these probes is to prove the
//! `TriggerCondition` → `TriggerEvent` mapping end to end, not merely that
//! `queue_damage_source_triggers` dispatches a hand-built `TriggeredAbilityDef`
//! correctly (that's `core::pb_dx36_deals_damage_roster`'s job).
//!
//! The noncombat "ping" in every probe here is a synthetic, engine-level
//! `ActivatedAbility` (`{T}: this creature deals N damage to target player`,
//! `Effect::DealDamage { source: None, .. }` so `ctx.source` — the creature
//! itself — becomes the damage source, CR 119.3) added directly to the pinging
//! creature's `ObjectSpec`. It carries no printed card and is not meant to
//! resemble one; it exists only to produce a `GameEvent::DamageDealt` whose
//! `source` is a battlefield creature, on demand, without depending on any
//! particular corpus card having that shape.

use std::collections::HashMap;

use mtg_engine::{
    all_cards, enrich_spec_from_def, process_command, AbilityDefinition, ActivatedAbility,
    ActivationCost, CardDefinition, CardEffectTarget, CardId, CardRegistry, CardType, Command,
    DamageRecipient, Effect, EffectAmount, GameEvent, GameState, GameStateBuilder, ObjectId,
    ObjectSpec, PlayerId, PlayerTarget, Step, SubType, Target, TargetRequirement, TriggerCondition,
    TypeLine, ZoneId,
};

use crate::pb_dp8_trigger_target_choice::answer_pending_trigger_targets;

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn p(n: u64) -> PlayerId {
    PlayerId(n)
}

fn find_object_opt(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    find_object_opt(state, name).unwrap_or_else(|| panic!("object '{name}' not found"))
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    state
        .objects()
        .iter()
        .filter(|(_, obj)| obj.zone == ZoneId::Hand(player))
        .count()
}

fn life_of(state: &GameState, player: PlayerId) -> i32 {
    state.players().get(&player).unwrap().life_total
}

/// Pass priority for all listed players once (resolves top of stack or advances step).
fn pass_all(state: GameState, players: &[PlayerId]) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    let mut current = state;
    for &pl in players {
        let (s, ev) = process_command(current, Command::PassPriority { player: pl })
            .unwrap_or_else(|e| panic!("PassPriority by {pl:?} failed: {e:?}"));
        current = s;
        all_events.extend(ev);
    }
    (current, all_events)
}

/// Pass priority repeatedly until the stack is empty or `limit` is reached — used
/// after answering a CR 603.3d target choice, since the trigger then still needs to
/// resolve.
fn pass_until_stack_empty(
    mut state: GameState,
    players: &[PlayerId],
    limit: usize,
) -> (GameState, Vec<GameEvent>) {
    let mut all_events = Vec::new();
    for _ in 0..limit {
        if state.stack_objects().is_empty() {
            break;
        }
        let (s, ev) = pass_all(state, players);
        state = s;
        all_events.extend(ev);
    }
    (state, all_events)
}

/// Declares `attacker_id` as an unblocked attacker on `p1`'s turn, targeting
/// `p2`, and advances through DeclareBlockers (`p2` declares none) into
/// CombatDamage. Returns the events from the final priority round, which is
/// where `GameEvent::CombatDamageDealt` (and anything it dispatches) appears.
fn attack_unblocked(
    state: GameState,
    p1: PlayerId,
    p2: PlayerId,
    attacker_id: ObjectId,
) -> (GameState, Vec<GameEvent>) {
    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "must reach DeclareBlockers"
    );
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![],
        },
    )
    .expect("DeclareBlockers failed");
    pass_all(state, &[p1, p2])
}

/// Untaps `id` directly. The ping ability's `{T}` cost taps its source; a combat
/// probe that reuses the same creature for both halves must untap it between
/// them (no untap STEP runs mid-test — CR 502.1 fires once per turn).
fn untap(state: &mut GameState, id: ObjectId) {
    if let Some(obj) = state.objects_mut().get_mut(&id) {
        obj.status.tapped = false;
    }
}

fn defs_of(def: &CardDefinition) -> HashMap<String, CardDefinition> {
    let mut m = HashMap::new();
    m.insert(def.name.clone(), def.clone());
    m
}

/// Looks a real corpus def up by name via `all_cards()` (SR-36 — never hand-transcribe).
fn corpus_def(name: &str) -> CardDefinition {
    all_cards()
        .into_iter()
        .find(|d| d.name == name)
        .unwrap_or_else(|| panic!("corpus def '{name}' not found by all_cards()"))
}

/// Builds an ObjectSpec for `def`, placed directly on the battlefield (never cast —
/// these probes are about trigger dispatch, not casting/attaching mechanics).
fn on_battlefield(player: PlayerId, name: &str, card_id: &str, def: &CardDefinition) -> ObjectSpec {
    enrich_spec_from_def(ObjectSpec::card(player, name), &defs_of(def))
        .with_card_id(CardId(card_id.to_string()))
        .in_zone(ZoneId::Battlefield)
}

/// Attaches `aura_name` to `creature_name` post-build (mirrors
/// `mechanics_e_l/enchant.rs`'s idiom — these Auras are placed directly on the
/// battlefield, never cast, so CR 303.4b's on-resolution attach never runs).
fn attach(state: &mut GameState, aura_name: &str, creature_name: &str) {
    let aura_id = find_object(state, aura_name);
    let creature_id = find_object(state, creature_name);
    if let Some(obj) = state.objects_mut().get_mut(&aura_id) {
        obj.attached_to = Some(creature_id);
    }
    if let Some(obj) = state.objects_mut().get_mut(&creature_id) {
        obj.attachments.push_back(aura_id);
    }
}

/// A synthetic Aura `CardDefinition` — "Enchant creature. Whenever enchanted
/// creature deals damage [to a player/an opponent], `effect`." — with the given
/// `combat_only`/`recipient`, routed through the REAL lowering.
fn synthetic_enchanted_damage_aura(
    name: &str,
    combat_only: bool,
    recipient: DamageRecipient,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(name.to_string()),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Enchantment].into_iter().collect(),
            subtypes: [SubType("Aura".to_string())].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Enchant creature\nWhenever enchanted creature deals damage, PB-DX36 test."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(mtg_engine::KeywordAbility::Enchant(
                mtg_engine::EnchantTarget::Creature,
            )),
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEnchantedCreatureDealsDamageToPlayer {
                    combat_only,
                    recipient,
                },
                effect,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}

/// A synthetic creature `CardDefinition` carrying `TriggerCondition::WhenDealsDamage`
/// (the self family), routed through the REAL lowering.
fn synthetic_self_damage_creature(
    name: &str,
    recipient: DamageRecipient,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        card_id: CardId(name.to_string()),
        name: name.to_string(),
        types: TypeLine {
            card_types: [CardType::Creature].into_iter().collect(),
            ..Default::default()
        },
        oracle_text: "Whenever this creature deals damage, PB-DX36 test.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WhenDealsDamage { recipient },
            effect,
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}

/// `{T}: this creature deals `amount` damage to target player.` — a synthetic,
/// engine-level activated ability (not going through the CardDefinition DSL),
/// added directly to a pinging creature's `ObjectSpec`. `source: None` means the
/// ability's own source (the creature) becomes `ctx.source` (CR 119.3), so the
/// resulting `GameEvent::DamageDealt.source` is the creature's `ObjectId`.
fn ping_ability(amount: i32) -> ActivatedAbility {
    ActivatedAbility {
        cost: ActivationCost {
            requires_tap: true,
            ..Default::default()
        },
        description: format!("{{T}}: This creature deals {amount} damage to target player."),
        effect: Some(Effect::DealDamage {
            source: None,
            target: CardEffectTarget::DeclaredTarget { index: 0 },
            amount: EffectAmount::Fixed(amount),
        }),
        sorcery_speed: false,
        targets: vec![TargetRequirement::TargetPlayer],
        activation_condition: None,
        activation_zone: None,
        once_per_turn: false,
        ..Default::default()
    }
}

fn ping_index(state: &GameState, pinger_id: ObjectId) -> usize {
    let obj = state.objects().get(&pinger_id).expect("pinger object");
    obj.characteristics.activated_abilities.len() - 1
}

/// Activates `pinger_id`'s LAST activated ability (the ping) targeting `target`.
/// Returns the state with the ability on the stack.
fn activate_ping(
    state: GameState,
    player: PlayerId,
    pinger_id: ObjectId,
    target: PlayerId,
) -> (GameState, Vec<GameEvent>) {
    let idx = ping_index(&state, pinger_id);
    process_command(
        state,
        Command::ActivateAbility {
            player,
            source: pinger_id,
            ability_index: idx,
            targets: vec![Target::Player(target)],
            discard_card: None,
            sacrifice_target: None,
            x_value: None,
            modes_chosen: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .unwrap_or_else(|e| panic!("ActivateAbility (ping) failed: {e:?}"))
}

/// Counts `AbilityTriggered` events whose `source_object_id == source`.
fn ability_triggered_count(events: &[GameEvent], source: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == source
            )
        })
        .count()
}

/// Counts the CR 603.3b dispatch signal for `source` in this event window --
/// `AbilityTriggered` (a slot with a forced or already-known answer, flushed
/// straight onto the stack) OR `TriggerTargetChoiceRequired` (a slot with a real
/// choice, suspended onto the CR 603.3d channel). The two are mutually
/// exclusive for one queued `PendingTrigger` within one flush: exactly one of
/// them fires per ability actually put in motion, which is what makes this the
/// right predicate for an "exactly once" count regardless of whether the
/// trigger's target happened to be forced or genuinely chosen.
fn dispatch_signal_count(events: &[GameEvent], source: ObjectId) -> usize {
    events
        .iter()
        .filter(|e| {
            matches!(
                e,
                GameEvent::AbilityTriggered { source_object_id, .. }
                if *source_object_id == source
            ) || matches!(
                e,
                GameEvent::TriggerTargetChoiceRequired { source_object_id, .. }
                if *source_object_id == source
            )
        })
        .count()
}

/// Two-player battlefield: `p1` controls `p1_creature` (with a ping ability
/// attached), `p2` controls TWO legal Sigil-of-Sleep targets (Bear, Bear2) so
/// the CR 603.3d target choice this batch's Auras issue is a genuine one --
/// `trigger_target_slot_forced_answer` only skips suspension when a slot has
/// exactly one candidate (or is optional with none), and a floor of ONE
/// candidate would leave the interactive channel itself unexercised.
fn base_state(
    p1_creature: ObjectSpec,
    extra_p1: Vec<ObjectSpec>,
    registry_defs: Vec<CardDefinition>,
) -> GameState {
    let p1 = p(1);
    let p2 = p(2);
    let bear = ObjectSpec::creature(p2, "Bear", 2, 2).in_zone(ZoneId::Battlefield);
    let bear2 = ObjectSpec::creature(p2, "Bear2", 2, 2).in_zone(ZoneId::Battlefield);
    let mut builder = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(registry_defs))
        .object(p1_creature)
        .object(bear)
        .object(bear2)
        .active_player(p1)
        .at_step(Step::PreCombatMain);
    for spec in extra_p1 {
        builder = builder.object(spec);
    }
    let mut state = builder.build().unwrap();
    state.turn_mut().priority_holder = Some(p1);
    state
}

// ─────────────────────────────────────────────────────────────────────────────
// t1 — Sigil of Sleep, NONCOMBAT damage, exactly once
// ─────────────────────────────────────────────────────────────────────────────

/// CR 510.3a / CR 603.2 / CR 603.2c — `Sigil of Sleep` (`Complete`, deck-legal)
/// fires from a NONcombat damage event, exactly once, and its effect executes:
/// the target creature that player controls returns to its owner's hand.
///
/// Before PB-DX36, `combat_only` was destructured away by the lowering and read
/// in exactly one place in the whole workspace (`state/hash.rs`); the dispatch
/// site only ever ran inside `GameEvent::CombatDamageDealt`, so this NONcombat
/// half was structurally unreachable (`OOS-CARDS2-6`).
#[test]
fn t1_sigil_of_sleep_fires_exactly_once_on_noncombat_damage() {
    let p1 = p(1);
    let p2 = p(2);
    let sigil = corpus_def("Sigil of Sleep");

    let pinger = ObjectSpec::creature(p1, "Pinger", 3, 3).with_activated_ability(ping_ability(2));
    let sigil_spec = on_battlefield(p1, "Sigil of Sleep", "sigil-of-sleep-t1", &sigil);

    let mut state = base_state(pinger, vec![sigil_spec], vec![sigil.clone()]);
    attach(&mut state, "Sigil of Sleep", "Pinger");

    let sigil_id = find_object(&state, "Sigil of Sleep");
    let pinger_id = find_object(&state, "Pinger");

    let (state, _) = activate_ping(state, p1, pinger_id, p2);
    let (state, resolve_events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        dispatch_signal_count(&resolve_events, sigil_id),
        1,
        "Sigil of Sleep must fire EXACTLY ONCE on a noncombat damage event, not \
         zero (the pre-PB-DX36 defect) and not more than one"
    );
    // CR 603.3d: with TWO legal creature targets (Bear, Bear2, both controlled
    // by the damaged player p2), the choice is a real one and suspends onto the
    // interactive channel PB-DP8 built, rather than being forced.
    let entry = state
        .pending_trigger_targets()
        .expect("Sigil of Sleep's trigger has two legal candidates and must suspend");
    assert_eq!(entry.slots.len(), 1);
    assert_eq!(
        entry.slots[0].candidates.len(),
        2,
        "both Bears must be legal candidates"
    );
    assert_eq!(
        state.stack_objects().len(),
        0,
        "the ping must have fully resolved"
    );
    let (state, _) = answer_pending_trigger_targets(state);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "answering CR 603.3d's choice must put the trigger on the stack"
    );

    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert!(
        state.stack_objects().is_empty(),
        "the trigger must have resolved"
    );

    assert_eq!(
        hand_count(&state, p2),
        1,
        "Sigil of Sleep's effect must return the named creature to its owner's hand"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t2 — Sigil of Sleep, COMBAT damage, exactly once
// ─────────────────────────────────────────────────────────────────────────────

/// CR 510.3a / CR 603.2c — the same trigger, fired by a COMBAT damage event.
/// This is the dedicated "exactly once" probe on the arm that fires BOTH the
/// `…CombatDamage…` and `…AnyDamage…` `TriggerEvent`s (execution notes §0.5(c)):
/// a `>= 1` assertion here would still pass on PB-DX47's double-push shape.
///
/// **What this probe does and does not cover** (`/review` HIGH 1): it drives
/// `attack_unblocked`, which produces exactly ONE `CombatDamageAssignment` for
/// the whole event (one attacker, one target player, no blockers). It proves
/// the two-`TriggerEvent`-family dispatch inside ONE call to
/// `queue_damage_source_triggers` is exactly-once. It proves NOTHING about
/// whether the CALLER dispatches once PER ASSIGNMENT rather than once per
/// SOURCE — a single-assignment fixture is structurally incapable of
/// distinguishing those two shapes, because they only diverge when a source
/// has more than one assignment (multi-block, trample). That is a real defect
/// this exact probe passed against for the whole implement phase (a bare
/// per-assignment dispatch loop, `dispatch_signal_count` still 1 here because
/// there is only ever one assignment to loop over). `t8`/`t9` below are the
/// multi-assignment probes.
#[test]
fn t2_sigil_of_sleep_fires_exactly_once_on_combat_damage() {
    let p1 = p(1);
    let p2 = p(2);
    let sigil = corpus_def("Sigil of Sleep");

    let attacker = ObjectSpec::creature(p1, "Attacker", 3, 3);
    let sigil_spec = on_battlefield(p1, "Sigil of Sleep", "sigil-of-sleep-t2", &sigil);

    let mut state = base_state(attacker, vec![sigil_spec], vec![sigil.clone()]);
    attach(&mut state, "Sigil of Sleep", "Attacker");
    // Combat needs DeclareAttackers, not PreCombatMain.
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p1);

    let sigil_id = find_object(&state, "Sigil of Sleep");
    let attacker_id = find_object(&state, "Attacker");

    let (state, damage_events) = attack_unblocked(state, p1, p2, attacker_id);

    assert_eq!(
        dispatch_signal_count(&damage_events, sigil_id),
        1,
        "Sigil of Sleep must fire EXACTLY ONCE on a combat damage event -- the \
         combat arm queues both the `…CombatDamage…` and `…AnyDamage…` `TriggerEvent` \
         families, and this variant's single `TriggeredAbilityDef` must only match \
         ONE of them (execution notes §0.5(c))"
    );
    let entry = state
        .pending_trigger_targets()
        .expect("Sigil of Sleep's trigger has two legal candidates and must suspend");
    assert_eq!(
        entry.slots[0].candidates.len(),
        2,
        "both Bears must be legal candidates"
    );
    let (state, _) = answer_pending_trigger_targets(state);
    assert_eq!(
        state.stack_objects().len(),
        1,
        "answering CR 603.3d's choice must put the trigger on the stack"
    );

    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert!(state.stack_objects().is_empty());
    assert_eq!(
        hand_count(&state, p2),
        1,
        "a Bear must have returned to p2's hand"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t3 — Exalted Angel: life gain equals the damage, combat and noncombat
// ─────────────────────────────────────────────────────────────────────────────

/// CR 603.2: `Exalted Angel` (`Complete`, this batch's ONE flip) gains life equal
/// to the damage IT dealt, on both a combat event and a noncombat one -- proving
/// `EffectAmount::DamageDealt` reads the actual triggering amount (not a fixed
/// value, not the creature's power) on both dispatch arms.
#[test]
fn t3_exalted_angel_gains_life_equal_to_damage_dealt_combat_and_noncombat() {
    let p1 = p(1);
    let p2 = p(2);
    let angel = corpus_def("Exalted Angel");

    let angel_spec = on_battlefield(p1, "Exalted Angel", "exalted-angel-t3", &angel)
        .with_activated_ability(ping_ability(3));

    let state = base_state(angel_spec, vec![], vec![angel.clone()]);
    let angel_id = find_object(&state, "Exalted Angel");

    let before = life_of(&state, p1);
    let (state, _) = activate_ping(state, p1, angel_id, p2);
    let (state, ping_events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ping_events, angel_id),
        1,
        "Exalted Angel's own trigger must fire exactly once on its ping"
    );
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        before + 3,
        "life gained must equal the NONCOMBAT damage amount (3), not the \
         creature's power (4)"
    );

    // Now attack for combat damage (power 4). Untap first: the ping's `{T}`
    // cost left it tapped, and no untap step ran in between.
    let mut state = state;
    untap(&mut state, angel_id);
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p1);
    let before2 = life_of(&state, p1);
    let (state, damage_events) = attack_unblocked(state, p1, p2, angel_id);
    assert_eq!(
        ability_triggered_count(&damage_events, angel_id),
        1,
        "Exalted Angel's trigger must fire exactly once on combat damage too"
    );
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        before2 + 4,
        "life gained must equal the COMBAT damage amount (4, the Angel's power)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t4 — combat_only discriminates
// ─────────────────────────────────────────────────────────────────────────────

/// CR 510.3a: `combat_only: false` fires on BOTH combat and noncombat damage --
/// the arm that was dead before PB-DX36 (`combat_only` was destructured away and
/// read nowhere but the hasher, so `sigil_of_sleep`'s `combat_only: false`
/// silently dropped its printed noncombat coverage).
#[test]
fn t4a_combat_only_false_fires_on_noncombat_damage() {
    let p1 = p(1);
    let p2 = p(2);
    let aura = synthetic_enchanted_damage_aura(
        "PB-DX36 Any-Damage Aura",
        false,
        DamageRecipient::Player,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );
    let pinger = ObjectSpec::creature(p1, "Pinger", 3, 3).with_activated_ability(ping_ability(1));
    let aura_spec = on_battlefield(p1, "PB-DX36 Any-Damage Aura", "pb-dx36-any-aura", &aura);

    let mut state = base_state(pinger, vec![aura_spec], vec![aura.clone()]);
    attach(&mut state, "PB-DX36 Any-Damage Aura", "Pinger");
    let aura_id = find_object(&state, "PB-DX36 Any-Damage Aura");
    let pinger_id = find_object(&state, "Pinger");

    let (state, _) = activate_ping(state, p1, pinger_id, p2);
    let (_state, events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&events, aura_id),
        1,
        "combat_only: false must fire on a NONcombat damage event"
    );
}

/// CR 510.3a: `combat_only: true` fires ONLY on combat damage, never on noncombat
/// -- the flag genuinely discriminates now, rather than being read nowhere.
#[test]
fn t4b_combat_only_true_does_not_fire_on_noncombat_damage() {
    let p1 = p(1);
    let p2 = p(2);
    let aura = synthetic_enchanted_damage_aura(
        "PB-DX36 Combat-Only Aura",
        true,
        DamageRecipient::Player,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );
    let pinger = ObjectSpec::creature(p1, "Pinger", 3, 3).with_activated_ability(ping_ability(1));
    let aura_spec = on_battlefield(p1, "PB-DX36 Combat-Only Aura", "pb-dx36-combat-aura", &aura);

    let mut state = base_state(pinger, vec![aura_spec], vec![aura.clone()]);
    attach(&mut state, "PB-DX36 Combat-Only Aura", "Pinger");
    let aura_id = find_object(&state, "PB-DX36 Combat-Only Aura");
    let pinger_id = find_object(&state, "Pinger");

    let (state, _) = activate_ping(state, p1, pinger_id, p2);
    let (state, events) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&events, aura_id),
        0,
        "combat_only: true must NOT fire on a NONcombat damage event"
    );

    // And now the combat half: same creature, same Aura, attack for combat
    // damage. Untap first: the ping's `{T}` cost left it tapped.
    let mut state = state;
    untap(&mut state, pinger_id);
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p1);
    let (_state, damage_events) = attack_unblocked(state, p1, p2, pinger_id);
    assert_eq!(
        ability_triggered_count(&damage_events, aura_id),
        1,
        "combat_only: true MUST fire on a combat damage event"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t5 — recipient discriminates
// ─────────────────────────────────────────────────────────────────────────────

/// CR 603.2 recipient clause: an `Opponent`-scoped `WhenDealsDamage` trigger does
/// NOT fire when the damaged player is the trigger source's own controller, and
/// DOES fire when the damaged player is a genuine opponent. A `Player`-scoped one
/// fires on both.
#[test]
fn t5_recipient_discriminates_opponent_from_player() {
    let p1 = p(1);
    let p2 = p(2);
    let player_creature = synthetic_self_damage_creature(
        "PB-DX36 Player Pinger",
        DamageRecipient::Player,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );
    let opponent_creature = synthetic_self_damage_creature(
        "PB-DX36 Opponent Pinger",
        DamageRecipient::Opponent,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );

    let player_spec = on_battlefield(
        p1,
        "PB-DX36 Player Pinger",
        "pb-dx36-player-pinger",
        &player_creature,
    )
    .with_activated_ability(ping_ability(1));
    let opponent_spec = on_battlefield(
        p1,
        "PB-DX36 Opponent Pinger",
        "pb-dx36-opponent-pinger",
        &opponent_creature,
    )
    .with_activated_ability(ping_ability(1));

    let state = base_state(
        player_spec,
        vec![opponent_spec],
        vec![player_creature.clone(), opponent_creature.clone()],
    );
    let player_id = find_object(&state, "PB-DX36 Player Pinger");
    let opponent_id = find_object(&state, "PB-DX36 Opponent Pinger");

    // (1) p1's own creatures ping p1 itself -- friendly-fire damage.
    let (state, _) = activate_ping(state, p1, player_id, p1);
    let (state, ev1) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ev1, player_id),
        1,
        "recipient: Player must fire on damage to ANY player, including its own controller"
    );

    let (state, _) = activate_ping(state, p1, opponent_id, p1);
    let (state, ev2) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ev2, opponent_id),
        0,
        "recipient: Opponent must NOT fire when the damaged player is the trigger \
         source's own controller"
    );

    // (2) now ping p2 -- a genuine opponent. Untap first: the previous ping
    // left this same creature tapped.
    let mut state = state;
    untap(&mut state, opponent_id);
    let (state, _) = activate_ping(state, p1, opponent_id, p2);
    let (_state, ev3) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ev3, opponent_id),
        1,
        "recipient: Opponent must fire when the damaged player IS an opponent of \
         the trigger source's controller"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t6 — EffectAmount::CombatDamageDealt vs EffectAmount::DamageDealt on a
// noncombat trigger
// ─────────────────────────────────────────────────────────────────────────────

/// Execution notes §0.5(d): `EffectAmount::CombatDamageDealt` reads
/// `ctx.combat_damage_amount`, which is 0 on a noncombat trigger; `DamageDealt`
/// reads `ctx.damage_dealt_amount`, the CR 608.2h/113.7a amount from the
/// triggering event whichever kind it was. The two are NOT redundant.
#[test]
fn t6_combat_damage_dealt_reads_zero_on_noncombat_while_damage_dealt_reads_the_amount() {
    let p1 = p(1);
    let p2 = p(2);
    let combat_amount_creature = synthetic_self_damage_creature(
        "PB-DX36 CombatAmount Pinger",
        DamageRecipient::Any,
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::CombatDamageDealt,
        },
    );
    let damage_amount_creature = synthetic_self_damage_creature(
        "PB-DX36 DamageAmount Pinger",
        DamageRecipient::Any,
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::DamageDealt,
        },
    );

    let combat_spec = on_battlefield(
        p1,
        "PB-DX36 CombatAmount Pinger",
        "pb-dx36-combatamount-pinger",
        &combat_amount_creature,
    )
    .with_activated_ability(ping_ability(5));
    let damage_spec = on_battlefield(
        p1,
        "PB-DX36 DamageAmount Pinger",
        "pb-dx36-damageamount-pinger",
        &damage_amount_creature,
    )
    .with_activated_ability(ping_ability(5));

    let state = base_state(
        combat_spec,
        vec![damage_spec],
        vec![
            combat_amount_creature.clone(),
            damage_amount_creature.clone(),
        ],
    );
    let combat_id = find_object(&state, "PB-DX36 CombatAmount Pinger");
    let damage_id = find_object(&state, "PB-DX36 DamageAmount Pinger");

    let life_before = life_of(&state, p1);
    let (state, _) = activate_ping(state, p1, combat_id, p2);
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        life_before,
        "EffectAmount::CombatDamageDealt must read 0 on a NONcombat trigger \
         (ctx.combat_damage_amount is only populated on the combat arm)"
    );

    let life_before2 = life_of(&state, p1);
    let (state, _) = activate_ping(state, p1, damage_id, p2);
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        life_before2 + 5,
        "EffectAmount::DamageDealt must read the actual amount (5) on a NONcombat trigger"
    );
    let _ = state;
}

// ─────────────────────────────────────────────────────────────────────────────
// t7 — DamageRecipient::Any and ::Player are equivalent on THIS variant
// ─────────────────────────────────────────────────────────────────────────────

/// Execution notes / plan step 2.1: on `WhenEnchantedCreatureDealsDamageToPlayer`,
/// `recipient: Any` and `recipient: Player` are stated to be equivalent, because
/// the dispatch site only ever fires this variant on damage to a player. Asserted
/// behaviourally, not merely commented: both fire on the same scenario.
#[test]
fn t7_any_and_player_recipient_are_equivalent_on_enchanted_creature_variant() {
    let p1 = p(1);
    let p2 = p(2);
    let any_aura = synthetic_enchanted_damage_aura(
        "PB-DX36 Any-Recipient Aura",
        false,
        DamageRecipient::Any,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );
    let player_aura = synthetic_enchanted_damage_aura(
        "PB-DX36 Player-Recipient Aura",
        false,
        DamageRecipient::Player,
        Effect::DrawCards {
            player: PlayerTarget::Controller,
            count: EffectAmount::Fixed(1),
        },
    );

    let any_pinger =
        ObjectSpec::creature(p1, "Any Pinger", 3, 3).with_activated_ability(ping_ability(1));
    let player_pinger =
        ObjectSpec::creature(p1, "Player Pinger", 3, 3).with_activated_ability(ping_ability(1));
    let any_aura_spec = on_battlefield(
        p1,
        "PB-DX36 Any-Recipient Aura",
        "pb-dx36-any-recipient",
        &any_aura,
    );
    let player_aura_spec = on_battlefield(
        p1,
        "PB-DX36 Player-Recipient Aura",
        "pb-dx36-player-recipient",
        &player_aura,
    );

    let mut state = base_state(
        any_pinger,
        vec![player_pinger, any_aura_spec, player_aura_spec],
        vec![any_aura.clone(), player_aura.clone()],
    );
    attach(&mut state, "PB-DX36 Any-Recipient Aura", "Any Pinger");
    attach(&mut state, "PB-DX36 Player-Recipient Aura", "Player Pinger");
    let any_aura_id = find_object(&state, "PB-DX36 Any-Recipient Aura");
    let player_aura_id = find_object(&state, "PB-DX36 Player-Recipient Aura");
    let any_pinger_id = find_object(&state, "Any Pinger");
    let player_pinger_id = find_object(&state, "Player Pinger");

    let (state, _) = activate_ping(state, p1, any_pinger_id, p2);
    let (state, ev1) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ev1, any_aura_id),
        1,
        "recipient: Any must fire on damage to a player"
    );

    let (state, _) = activate_ping(state, p1, player_pinger_id, p2);
    let (_state, ev2) = pass_all(state, &[p1, p2]);
    assert_eq!(
        ability_triggered_count(&ev2, player_aura_id),
        1,
        "recipient: Player must fire identically on the same scenario"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t8 — multi-block: self family fires ONCE with the SUMMED amount (`/review` HIGH 1)
// ─────────────────────────────────────────────────────────────────────────────

/// CR 510.2 / CR 603.2c (`/review` HIGH 1) — a source with MORE THAN ONE
/// combat damage assignment in the same event must dispatch the self family
/// (`TriggerEvent::SelfDealsDamage`) EXACTLY ONCE for the whole event, with
/// `amount` = the SUM of every assignment — not once PER ASSIGNMENT, each
/// carrying only that one assignment's own amount. A 5/5 (no trample) blocked
/// by two 2/2s assigns 2 (lethal) to the first blocker in order, then the
/// remaining 3 to the second (CR 510.1c) — TWO `CombatDamageAssignment`s in
/// ONE `GameEvent::CombatDamageDealt`, both targeting a CREATURE (neither a
/// Player), so `damaged_player` is `None` throughout: this fixture is
/// deliberately shaped to isolate the self-family regression from t9's
/// aura/trample shape, which also exercises a Player-target assignment and the
/// attachment family.
///
/// Pre-fix, this dispatched `SelfDealsDamage` twice (once per assignment) with
/// `amount` 2 then 3 — `dispatch_signal_count` reads 2 (should be 1) and life
/// is gained as two separate +2/+3 resolutions rather than one +5 resolution
/// (proven live on `exalted_angel`, the flip this batch's coverage prediction
/// named). Reverting to that per-assignment dispatch loop is proven to redden
/// this exact test (see `memory/primitives/pb-DX36-execution-notes.md`'s
/// `/review` fix-cycle notes for the executed revert).
#[test]
fn t8_multi_blocker_self_family_fires_once_with_the_summed_amount() {
    let p1 = p(1);
    let p2 = p(2);

    let mut attacker_def = synthetic_self_damage_creature(
        "MultiBlock Attacker",
        DamageRecipient::Any,
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::DamageDealt,
        },
    );
    attacker_def.power = Some(5);
    attacker_def.toughness = Some(5);

    let attacker = on_battlefield(
        p1,
        "MultiBlock Attacker",
        "pb-dx36-multiblock",
        &attacker_def,
    );

    let mut state = base_state(attacker, vec![], vec![attacker_def.clone()]);
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p1);

    let attacker_id = find_object(&state, "MultiBlock Attacker");
    let bear1_id = find_object(&state, "Bear");
    let bear2_id = find_object(&state, "Bear2");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "must reach DeclareBlockers"
    );

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(bear1_id, attacker_id), (bear2_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers failed");
    // p1 orders blockers: Bear first, then Bear2 (mirrors
    // `combat::test_702_19b_trample_multiple_blockers_excess_to_player`'s idiom).
    let (state, _) = process_command(
        state,
        Command::OrderBlockers {
            player: p1,
            attacker: attacker_id,
            order: vec![bear1_id, bear2_id],
        },
    )
    .expect("OrderBlockers failed");

    let before = life_of(&state, p1);
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        dispatch_signal_count(&damage_events, attacker_id),
        1,
        "the self family must fire EXACTLY ONCE for this event, not once per \
         assignment (2 assignments here: 2 to Bear, 3 to Bear2) -- the shape \
         `/review` HIGH 1 reproduced live on exalted_angel"
    );
    // The trigger is untargeted (no CR 603.3d suspension), so it lands
    // straight on the stack via `AbilityTriggered` -- it still needs a
    // further priority round to RESOLVE (mirrors `t3`'s idiom).
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        before + 5,
        "life gained must equal the EVENT TOTAL (2 + 3 = 5), not one \
         assignment's amount resolved twice (2 then 3, same eventual total but \
         as TWO separate trigger resolutions -- observably different for any \
         card whose effect is not simply additive, e.g. \"exile the top card\" \
         would exile 2 cards instead of 1)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// t9 — trample: self family fires ONCE with the summed amount; aura unaffected
// ─────────────────────────────────────────────────────────────────────────────

/// CR 510.2 / CR 603.2c / CR 702.19b (`/review` HIGH 1) — a 6/6 TRAMPLER
/// carrying an attached Aura (`Sigil of Sleep`, real corpus `Complete` def) is
/// blocked by ONE 2/2: assigns 2 (lethal) to the blocker and 4 (trample) to
/// the defending player — again TWO assignments in one event, one
/// Creature-target and one Player-target. The self family must fire ONCE with
/// `amount` 6 (2 + 4, the event total, regardless of recipient — CR 603.2's
/// "any damage"). The ATTACHMENT (Aura) family must stay at count == 1 with
/// its pre-existing amount semantics UNCHANGED — the Player-target
/// assignment's OWN amount (4), never the summed total (6) — because that half
/// was ALREADY correct before this fix (only the Player-target assignment
/// ever populated `damaged_player`, so only it ever reached the attachment
/// dispatch). This probe pins BOTH halves together so a future change to the
/// self-family fix cannot silently widen the attachment family's amount too.
#[test]
fn t9_trample_self_family_fires_once_aura_family_unaffected() {
    let p1 = p(1);
    let p2 = p(2);
    let sigil = corpus_def("Sigil of Sleep");

    let mut trampler_def = synthetic_self_damage_creature(
        "Trample Attacker",
        DamageRecipient::Any,
        Effect::GainLife {
            player: PlayerTarget::Controller,
            amount: EffectAmount::DamageDealt,
        },
    );
    trampler_def.power = Some(6);
    trampler_def.toughness = Some(6);

    let attacker = on_battlefield(p1, "Trample Attacker", "pb-dx36-trample", &trampler_def)
        .with_keyword(mtg_engine::KeywordAbility::Trample);
    let sigil_spec = on_battlefield(p1, "Sigil of Sleep", "sigil-of-sleep-t9", &sigil);

    let mut state = base_state(
        attacker,
        vec![sigil_spec],
        vec![trampler_def.clone(), sigil.clone()],
    );
    attach(&mut state, "Sigil of Sleep", "Trample Attacker");
    state.turn_mut().step = Step::DeclareAttackers;
    state.turn_mut().priority_holder = Some(p1);

    let attacker_id = find_object(&state, "Trample Attacker");
    let sigil_id = find_object(&state, "Sigil of Sleep");
    let bear1_id = find_object(&state, "Bear");
    // Only Bear blocks -- Bear2 stays an unused-but-legal candidate for
    // Sigil of Sleep's own target choice (both Bears are controlled by the
    // damaged player p2). A second BLOCKER would add a third assignment and
    // conflate this probe with t8's multi-blocker shape.

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p1,
            attackers: vec![(attacker_id, mtg_engine::AttackTarget::Player(p2))],
            enlist_choices: vec![],
            exert_choices: vec![],
            hybrid_choices: vec![],
            phyrexian_life_payments: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p1, p2]);
    assert_eq!(
        state.turn().step,
        Step::DeclareBlockers,
        "must reach DeclareBlockers"
    );

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p2,
            blockers: vec![(bear1_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers failed");

    let before = life_of(&state, p1);
    let (state, damage_events) = pass_all(state, &[p1, p2]);

    assert_eq!(
        dispatch_signal_count(&damage_events, attacker_id),
        1,
        "the self family must fire EXACTLY ONCE for this event (2 to the \
         blocker, 4 trample to p2 -- two assignments, one call)"
    );
    assert_eq!(
        dispatch_signal_count(&damage_events, sigil_id),
        1,
        "the Aura (attachment) family must stay at EXACTLY ONE -- it was \
         already correct before this fix and must not be perturbed by the \
         self-family grouping"
    );
    // Bear (the blocker) took exactly LETHAL damage (2, its full toughness) --
    // CR 704.5g destroys it as an SBA before the trigger's targets are
    // evaluated, so Sigil of Sleep's target slot has only ONE legal candidate
    // left (Bear2) and is FORCED rather than suspended (`t1`/`t2` cover the
    // genuine-choice/suspension path on a noncombat and an unblocked-combat
    // event respectively; this probe's subject is the count/amount fix, not
    // the suspension mechanic, so both triggers -- the untargeted self family
    // and the now-forced Aura family -- land straight on the stack and
    // resolve on their own).
    let (state, _) = pass_until_stack_empty(state, &[p1, p2], 8);
    assert_eq!(
        life_of(&state, p1),
        before + 6,
        "life gained must equal the EVENT TOTAL (2 + 4 = 6, the trampler's \
         full power), not the player-target assignment's amount alone (4)"
    );
    assert_eq!(
        hand_count(&state, p2),
        1,
        "Sigil of Sleep's effect must still return the surviving Bear (Bear2) \
         to p2's hand"
    );
}
