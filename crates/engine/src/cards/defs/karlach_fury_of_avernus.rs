// Karlach, Fury of Avernus -- {4}{R}, Legendary Creature -- Tiefling Barbarian 5/4
// Whenever you attack, if it's the first combat phase of the turn, untap all attacking
// creatures, grant first strike until end of turn, add an additional combat phase.
// Choose a Background.
//
// NOTE: "Whenever you attack" means whenever you (the player) declare attackers.
// The engine models this as WhenAttacks on Karlach itself, which is a known
// simplification (DSL gap: no WhenYouDeclareAttackers condition). Karlach must
// personally attack for the trigger to fire.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("karlach-fury-of-avernus"),
        name: "Karlach, Fury of Avernus".to_string(),
        mana_cost: Some(ManaCost { generic: 4, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Tiefling", "Barbarian"],
        ),
        oracle_text: "Whenever you attack, if it's the first combat phase of the turn, untap all attacking creatures. They gain first strike until end of turn. After this phase, there is an additional combat phase.\nChoose a Background (You can have a Background as a second commander.)".to_string(),
        power: Some(5),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::ChooseABackground),
            // Whenever Karlach attacks (first combat phase only):
            // 1. Untap all attacking creatures.
            // 2. Each attacking creature gains first strike until end of turn.
            // 3. After this phase, there is an additional combat phase.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                intervening_if: Some(Condition::IsFirstCombatPhase),
                effect: Effect::Sequence(vec![
                    // Untap all attacking creatures.
                    Effect::ForEach {
                        over: ForEachTarget::EachAttackingCreature,
                        effect: Box::new(Effect::UntapPermanent {
                            target: EffectTarget::DeclaredTarget { index: 0 },
                        }),
                    },
                    // Grant first strike to all attacking creatures until end of turn.
                    Effect::ForEach {
                        over: ForEachTarget::EachAttackingCreature,
                        effect: Box::new(Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::Ability,
                                modification: LayerModification::AddKeyword(
                                    KeywordAbility::FirstStrike,
                                ),
                                filter: EffectFilter::DeclaredTarget { index: 0 },
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        }),
                    },
                    // After this phase, there is an additional combat phase.
                    Effect::AdditionalCombatPhase { followed_by_main: false },
                ]),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
