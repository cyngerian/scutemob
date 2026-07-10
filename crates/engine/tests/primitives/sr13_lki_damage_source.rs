//! SR-13: a damage source's characteristics must use last known information.
//!
//! CR 702.80c: "The wither rules function no matter what zone an object with wither
//! deals damage from." CR 702.90e says the same for infect. CR 608.2h / 113.7a: an
//! effect that references information about its source uses the source's last known
//! information once the source is no longer in the zone it is expected to be in — and
//! "the source can still perform the action even though it no longer exists."
//!
//! CR 400.7 retires a source's `ObjectId` on any zone change, so a live
//! `calculate_characteristics(source)` returns `None` once the source has died. Before
//! SR-13 the `Effect::DealDamage` path read the source's wither/infect keywords live and
//! silently treated a dead source as having neither (discovered during SR-4). These tests
//! drive the exact scenario the criterion names — a source that dies to state-based
//! actions while its "deals damage" ability is still on the stack — and assert the
//! keyword still applies, through the LKI snapshot captured in `move_object_to_zone`.
//!
//! Mechanism (mirrors `mechanics_e_l/enrage.rs`): a creature with an Enrage-style
//! "whenever this is dealt damage, it deals damage …" trigger (`SelfIsDealtDamage`) is
//! dealt lethal combat damage. The trigger fires, SBAs kill the creature, and the trigger
//! — carrying its embedded effect (MR-B12-04) and naming the now-dead creature as its
//! source — resolves afterward. The damage is therefore always dealt by a source that has
//! already left the battlefield.

use mtg_engine::{
    process_command, AttackTarget, CardEffectTarget, CardRegistry, Command, ContinuousEffect,
    CounterType, Effect, EffectAmount, EffectDuration, EffectFilter, EffectId, EffectLayer,
    GameEvent, GameState, GameStateBuilder, KeywordAbility, LayerModification, ObjectId,
    ObjectSpec, PlayerId, Step, TriggerEvent, TriggeredAbilityDef, ZoneId,
};

// ── Helpers ─────────────────────────────────────────────────────────────────────

fn find_object_opt(state: &GameState, name: &str) -> Option<ObjectId> {
    state
        .objects()
        .iter()
        .find(|(_, obj)| obj.characteristics.name == name)
        .map(|(id, _)| *id)
}

fn find_object(state: &GameState, name: &str) -> ObjectId {
    find_object_opt(state, name).unwrap_or_else(|| panic!("object '{}' not found", name))
}

/// `true` if a specific `ObjectId` has been retired (removed from `state.objects`), i.e. the
/// object it named has changed zones (CR 400.7). Note: after a creature dies, a *new* object
/// with the same name exists in the graveyard, so name-based lookup would find that copy —
/// this checks the exact pre-death id instead.
fn id_is_retired(state: &GameState, id: ObjectId) -> bool {
    state.objects().get(&id).is_none()
}

/// Pass priority for all listed players once (resolves top of stack or advances step).
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

/// A 0/3 blocker with an Enrage-style "whenever this is dealt damage, it deals `amount`
/// damage to `target`" trigger. Power 0 so its own combat damage never marks the attacker,
/// keeping the trigger's LKI damage the only variable under test. `keywords` are the
/// printed keywords (e.g. `Infect`) — `None` for the layer-granted test.
fn enrage_pinger(
    owner: PlayerId,
    name: &str,
    keywords: &[KeywordAbility],
    target: CardEffectTarget,
    amount: i32,
) -> ObjectSpec {
    let mut spec =
        ObjectSpec::creature(owner, name, 0, 3).with_triggered_ability(TriggeredAbilityDef {
            counter_filter: None,
            counter_on_self: false,
            once_per_turn: false,
            etb_filter: None,
            death_filter: None,
            combat_damage_filter: None,
            triggering_creature_filter: None,
            targets: vec![],
            trigger_on: TriggerEvent::SelfIsDealtDamage,
            intervening_if: None,
            description: format!("Whenever ~ is dealt damage, it deals {} damage.", amount),
            effect: Some(Effect::DealDamage {
                target,
                amount: EffectAmount::Fixed(amount),
            }),
        });
    for kw in keywords {
        spec = spec.with_keyword(kw.clone());
    }
    spec
}

/// Common setup: P2 attacks P1 with a big creature; P1 blocks with `pinger`, which takes
/// lethal combat damage. Returns the post-combat state (pinger dead, its trigger on the
/// stack) plus the attacker's id.
fn run_lethal_block(pinger: ObjectSpec, attacker_power: i32) -> (GameState, ObjectId) {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let attacker = ObjectSpec::creature(p2, "Big Attacker", attacker_power, attacker_power);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(pinger)
        .object(attacker)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Big Attacker");
    let pinger_id = find_object(&state, "Pinger");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p2, p1]);

    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p1,
            blockers: vec![(pinger_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers failed");
    // Advance through the combat damage step: damage is dealt, the Enrage trigger fires,
    // SBAs kill the 0/3 pinger, and the trigger is put on the stack.
    let (state, _events) = pass_all(state, &[p2, p1]);

    assert!(
        id_is_retired(&state, pinger_id),
        "SR-13 precondition: the pinger must be dead (CR 400.7 retired its ObjectId) \
         before its damage ability resolves"
    );
    assert_eq!(
        state.stack_objects().len(),
        1,
        "SR-13 precondition: the pinger's damage trigger must be on the stack after it dies"
    );
    (state, attacker_id)
}

// ── Test 1: Infect from a dead source still gives poison (CR 702.90e) ──────────────

#[test]
/// CR 702.90e / 702.90b / 608.2h / 113.7a: an infect source that has left the battlefield
/// still deals its damage as poison counters, not life loss. The discriminator against the
/// pre-SR-13 bug is precise: the bug treated the dead source as having no infect, so the
/// opponent would have *lost life* instead of gaining poison.
fn test_infect_source_dies_with_damage_ability_on_stack() {
    let p2 = PlayerId(2);
    let pinger = enrage_pinger(
        PlayerId(1),
        "Pinger",
        &[KeywordAbility::Infect],
        CardEffectTarget::EachOpponent,
        3,
    );
    let (state, _attacker_id) = run_lethal_block(pinger, 5);
    let life_before = state.players()[&p2].life_total;

    // Resolve the trigger: the dead infect source deals 3 damage to P2.
    let (state, _) = pass_all(state, &[p2, PlayerId(1)]);

    assert_eq!(
        state.players()[&p2].poison_counters,
        3,
        "CR 702.90e/608.2h: a dead infect source must still give poison counters via LKI"
    );
    assert_eq!(
        state.players()[&p2].life_total,
        life_before,
        "CR 702.90b: infect damage causes no life loss — proving the source was NOT treated \
         as having lost its infect after leaving the battlefield"
    );
}

// ── Test 2: Wither from a dead source still gives -1/-1 counters (CR 702.80c) ──────

#[test]
/// CR 702.80c / 702.80a / 608.2h / 113.7a: a wither source that has left the battlefield
/// still deals its damage as -1/-1 counters, not marked damage.
fn test_wither_source_dies_with_damage_ability_on_stack() {
    let pinger = enrage_pinger(
        PlayerId(1),
        "Pinger",
        &[KeywordAbility::Wither],
        CardEffectTarget::AllCreatures,
        3,
    );
    // Attacker is 5/5; after 3 -1/-1 counters it is a 2/2 with 0 marked damage — survives,
    // so we can inspect its counters (the only creature left once the pinger has died).
    let (state, attacker_id) = run_lethal_block(pinger, 5);

    let (state, _) = pass_all(state, &[PlayerId(2), PlayerId(1)]);

    let attacker = state
        .objects()
        .get(&attacker_id)
        .expect("attacker should survive 3 -1/-1 counters (5/5 -> 2/2)");
    assert_eq!(
        attacker
            .counters
            .get(&CounterType::MinusOneMinusOne)
            .copied()
            .unwrap_or(0),
        3,
        "CR 702.80c/608.2h: a dead wither source must still place -1/-1 counters via LKI"
    );
    assert_eq!(
        attacker.damage_marked, 0,
        "CR 120.3d: wither damage must NOT mark damage — proving the dead source kept its wither"
    );
}

// ── Test 3: Deathtouch from a dead source is still lethal (CR 702.2b) ──────────────

#[test]
/// CR 702.2b / 608.2h: any nonzero damage from a source with deathtouch is lethal. A
/// deathtouch source that has left the battlefield still marks its target as having taken
/// lethal damage, so the target dies to SBAs even though only 1 damage was dealt.
fn test_deathtouch_source_dies_with_damage_ability_on_stack() {
    let pinger = enrage_pinger(
        PlayerId(1),
        "Pinger",
        &[KeywordAbility::Deathtouch],
        CardEffectTarget::AllCreatures,
        1,
    );
    // Attacker 5/5 takes only 1 damage — lethal ONLY because of deathtouch.
    let (state, attacker_id) = run_lethal_block(pinger, 5);

    let (state, _) = pass_all(state, &[PlayerId(2), PlayerId(1)]);

    assert!(
        state.objects().get(&attacker_id).is_none()
            || matches!(
                state.objects().get(&attacker_id).map(|o| o.zone),
                Some(ZoneId::Graveyard(_))
            ),
        "CR 702.2b/608.2h: a dead deathtouch source must still deal lethal damage via LKI \
         (1 damage to a 5/5 kills it only through deathtouch)"
    );
}

// ── Test 4: Lifelink from a dead source still gains life (CR 702.15b) ──────────────

#[test]
/// CR 702.15a/b / 608.2h: a lifelink source that has left the battlefield still causes its
/// controller to gain life equal to the damage dealt — using the controller from the
/// source's last known information, since the source object no longer exists.
fn test_lifelink_source_dies_with_damage_ability_on_stack() {
    let p1 = PlayerId(1);
    let pinger = enrage_pinger(
        p1,
        "Pinger",
        &[KeywordAbility::Lifelink],
        CardEffectTarget::EachOpponent,
        4,
    );
    let (state, _attacker_id) = run_lethal_block(pinger, 5);
    let life_before = state.players()[&p1].life_total;

    let (state, _) = pass_all(state, &[PlayerId(2), p1]);

    assert_eq!(
        state.players()[&p1].life_total,
        life_before + 4,
        "CR 702.15b/608.2h: the dead lifelink source's controller must still gain life via LKI"
    );
}

// ── Test 5: LKI is layer-resolved, not just the printed keyword ───────────────────

#[test]
/// Layer-granted infect, granted BEFORE the source dies: the definitive test that the LKI
/// snapshot is layer-resolved. Infect comes from a continuous effect on a separate anthem;
/// the pinger has none printed. The anthem is still on the battlefield when the pinger dies,
/// so `calculate_characteristics` (run at capture time in `move_object_to_zone`) sees the
/// grant and the snapshot carries infect.
fn test_granted_infect_present_before_death() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    let attacker_power = 5;
    let attacker = ObjectSpec::creature(p2, "Big Attacker", attacker_power, attacker_power);
    let pinger = enrage_pinger(p1, "Pinger", &[], CardEffectTarget::EachOpponent, 3);
    let anthem = ObjectSpec::creature(p1, "Infect Anthem", 1, 1);

    let mut state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(pinger)
        .object(attacker)
        .object(anthem)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Big Attacker");
    let pinger_id = find_object(&state, "Pinger");
    let anthem_id = find_object(&state, "Infect Anthem");

    // Grant infect to P1's creatures while everyone is still on the battlefield.
    state.continuous_effects_mut().push_back(ContinuousEffect {
        id: EffectId(9002),
        source: Some(anthem_id),
        timestamp: 1,
        layer: EffectLayer::Ability,
        duration: EffectDuration::WhileSourceOnBattlefield,
        filter: EffectFilter::CreaturesYouControl,
        modification: LayerModification::AddKeyword(KeywordAbility::Infect),
        is_cda: false,
        condition: None,
    });
    // The grant is live now (base characteristics do NOT carry it).
    assert!(
        mtg_engine::calculate_characteristics(&state, pinger_id)
            .map(|c| c.keywords.contains(&KeywordAbility::Infect))
            .unwrap_or(false),
        "precondition: the pinger has infect only via the layer system, not printed"
    );

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p2, p1]);
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p1,
            blockers: vec![(pinger_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers failed");
    let (state, _) = pass_all(state, &[p2, p1]);

    assert!(
        id_is_retired(&state, pinger_id),
        "precondition: the pinger died with its trigger on the stack"
    );
    let life_before = state.players()[&p2].life_total;
    let (state, _) = pass_all(state, &[p2, p1]);

    assert_eq!(
        state.players()[&p2].poison_counters,
        3,
        "CR 604.3/608.2h: layer-granted infect must survive the source's death via a \
         layer-resolved LKI snapshot"
    );
    assert_eq!(
        state.players()[&p2].life_total,
        life_before,
        "layer-granted infect on a dead source must still convert life loss to poison"
    );
}

// ── Test 6: An alive source is unaffected (regression / non-vacuity control) ───────

#[test]
/// Control: when the source does NOT leave its zone, behaviour is unchanged — infect is read
/// live and still applies. Confirms the LKI fallback did not alter the alive-source path.
fn test_alive_infect_source_unchanged() {
    let p1 = PlayerId(1);
    let p2 = PlayerId(2);
    // A 0/6 pinger blocks a 5/5: it survives combat, so when its trigger resolves the source
    // is still on the battlefield and infect is read live.
    let pinger = enrage_pinger(
        p1,
        "Pinger",
        &[KeywordAbility::Infect],
        CardEffectTarget::EachOpponent,
        3,
    );
    let pinger = {
        // Rebuild with toughness 6 so it survives 5 damage.
        let mut s = ObjectSpec::creature(p1, "Pinger", 0, 6);
        for tr in pinger.triggered_abilities {
            s = s.with_triggered_ability(tr);
        }
        s.with_keyword(KeywordAbility::Infect)
    };
    let attacker = ObjectSpec::creature(p2, "Big Attacker", 5, 5);

    let state = GameStateBuilder::new()
        .add_player(p1)
        .add_player(p2)
        .with_registry(CardRegistry::new(vec![]))
        .object(pinger)
        .object(attacker)
        .active_player(p2)
        .at_step(Step::DeclareAttackers)
        .build()
        .unwrap();

    let attacker_id = find_object(&state, "Big Attacker");
    let pinger_id = find_object(&state, "Pinger");

    let (state, _) = process_command(
        state,
        Command::DeclareAttackers {
            player: p2,
            attackers: vec![(attacker_id, AttackTarget::Player(p1))],
            enlist_choices: vec![],
            exert_choices: vec![],
        },
    )
    .expect("DeclareAttackers failed");
    let (state, _) = pass_all(state, &[p2, p1]);
    let (state, _) = process_command(
        state,
        Command::DeclareBlockers {
            player: p1,
            blockers: vec![(pinger_id, attacker_id)],
        },
    )
    .expect("DeclareBlockers failed");
    let (state, _) = pass_all(state, &[p2, p1]);

    assert!(
        find_object_opt(&state, "Pinger").is_some(),
        "precondition: the 0/6 pinger should survive 5 combat damage"
    );
    let life_before = state.players()[&p2].life_total;
    let (state, _) = pass_all(state, &[p2, p1]);

    assert_eq!(
        state.players()[&p2].poison_counters,
        3,
        "an alive infect source applies poison via the live read path (unchanged by SR-13)"
    );
    assert_eq!(
        state.players()[&p2].life_total,
        life_before,
        "alive infect source: still no life loss"
    );
}
