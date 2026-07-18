// Scute Swarm — {2}{G}, Creature — Insect 1/1
// Landfall — Whenever a land you control enters, create a 1/1 green Insect creature token.
// If you control six or more lands, create a token that's a copy of this creature instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scute-swarm"),
        name: "Scute Swarm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            green: 1,
            ..Default::default()
        }),
        types: creature_types(&["Insect"]),
        oracle_text: "Landfall \u{2014} Whenever a land you control enters, create a 1/1 green \
                      Insect creature token. If you control six or more lands, create a token \
                      that's a copy of this creature instead."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                filter: Some(TargetFilter {
                    has_card_type: Some(CardType::Land),
                    controller: TargetController::You,
                    ..Default::default()
                }),
                exclude_self: false,
            },
            effect: Effect::Conditional {
                condition: Condition::YouControlNOrMoreWithFilter {
                    count: 6,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                },
                if_true: Box::new(Effect::CreateTokenCopy {
                    source: EffectTarget::Source,
                    enters_tapped_and_attacking: false,
                    except_not_legendary: false,
                    gains_haste: false,
                    delayed_action: None,
                }),
                if_false: Box::new(Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Insect".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Insect".to_string())].into_iter().collect(),
                        colors: [Color::Green].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: imbl::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                }),
            },
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        ..Default::default()
    }
}
