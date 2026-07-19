// Loyal Apprentice — {1}{R}, Creature — Human Artificer 2/1
// Haste
// Lieutenant — At the beginning of combat on your turn, if you control your commander,
// create a 1/1 colorless Thopter artifact creature token with flying. That token gains
// haste until end of turn.
//
// PB-OS9 / CR 903.3d: Lieutenant's condition half is now expressible --
// Condition::YouControlYourCommander is wired as an intervening-if on an
// AtBeginningOfCombat trigger (CR 603.4 resolution-time check). Token haste-until-EOT:
// the DSL has no ApplyContinuousEffect target scoped to "the permanent I just created"
// (EffectFilter has no LastCreatedPermanent-equivalent variant; only EffectTarget does,
// used by AttachEquipment/MoveZone-style effects, not ContinuousEffectDef.filter). A
// permanent-haste TokenSpec.keywords entry is the accepted functionally-equivalent
// fallback (a token's haste is unobservable after the turn it is created -- it loses
// summoning sickness anyway; same pattern as legion_warboss.rs).
//
// STILL BLOCKED (newly discovered during PB-OS9, verified by execution --
// probe_at_beginning_of_combat, PB-OS9 runner session): `TriggerCondition::
// AtBeginningOfCombat` has NO card-def sweep anywhere in the engine.
// `crates/engine/src/rules/turn_actions.rs` has a hardcoded per-step sweep for
// AtBeginningOfYourUpkeep, AtBeginningOfFirstMainPhase, AtBeginningOfPostcombatMain,
// and AtBeginningOfYourEndStep (each scans battlefield objects' card-registry
// abilities and pushes a PendingTrigger) -- but `begin_combat()` (the
// Step::BeginningOfCombat handler) only queues EMBLEM triggers
// (`collect_emblem_triggers_for_event`), never card-defined `AbilityDefinition::
// Triggered { trigger_condition: TriggerCondition::AtBeginningOfCombat, .. }`
// abilities. Confirmed empirically: transitioning a battlefield object with this
// trigger condition into BeginningOfCombat produces ZERO pending triggers and ZERO
// stack objects. This is a pre-existing engine gap (also silently affects
// legion_warboss.rs, goblin_rabblemaster.rs, mirage_phalanx.rs, helm_of_the_host.rs --
// out of PB-OS9 scope to touch), not something PB-OS9's plan anticipated or scoped.
// The DSL below is the CR-correct translation and is ready to fire once that sweep is
// added; it is inert (never queued) until then, so this does NOT produce wrong game
// state -- just incomplete. Flagged as a new seed for a future PB (recommend:
// "card-def AtBeginningOfCombat sweep in begin_combat()").
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("loyal-apprentice"),
        name: "Loyal Apprentice".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Artificer"]),
        oracle_text: "Haste\nLieutenant — At the beginning of combat on your turn, if you control \
                      your commander, create a 1/1 colorless Thopter artifact creature token with \
                      flying. That token gains haste until end of turn."
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // Lieutenant — CR 903.3d: "At the beginning of combat on your turn, if you
            // control your commander, create a 1/1 colorless Thopter artifact creature
            // token with flying. That token gains haste until end of turn."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Thopter".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature]
                            .into_iter()
                            .collect(),
                        subtypes: [SubType("Thopter".to_string())].into_iter().collect(),
                        colors: imbl::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Flying, KeywordAbility::Haste]
                            .into_iter()
                            .collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: Some(Condition::YouControlYourCommander),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "PB-OS9: the Lieutenant CONDITION half is now correctly modeled -- intervening_if: \
             Condition::YouControlYourCommander on the AtBeginningOfCombat trigger (CR \
             903.3d/603.4). STILL BLOCKED: TriggerCondition::AtBeginningOfCombat has no card-def \
             sweep anywhere in the engine (begin_combat() only queues emblem triggers), confirmed \
             by execution -- the trigger never queues, so this ability is currently inert (not \
             wrong game state, just non-firing). Pre-existing engine gap, also affects \
             legion_warboss/goblin_rabblemaster/mirage_phalanx/helm_of_the_host. Token spec \
             (Thopter, flying, permanent-haste fallback) is correct and ready.",
        ),
        ..Default::default()
    }
}
