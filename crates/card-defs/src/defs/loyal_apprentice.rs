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
// PB-RS3: the sweep gap is CLOSED -- `begin_combat` (turn_actions.rs) now scans the
// battlefield for card-def AtBeginningOfCombat triggers (in addition to the
// pre-existing emblem-only scan), so this ability fires. Flipped to Complete.
//
// PB-DP6 (`scutemob-154`): queue-time gate added (CR 603.4's other half); both
// halves of CR 603.4 now hold. The divergent case previously documented here (you
// do NOT control your commander at beginning of combat but regain control before
// the trigger resolves) is fixed -- the trigger is no longer queued at all when
// the condition is false at the moment of the event.
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
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
