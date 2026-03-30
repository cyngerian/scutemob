// Haywire Mite
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("haywire-mite"),
        name: "Haywire Mite".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Insect"]),
        oracle_text: "When this creature dies, you gain 2 life.
{G}, Sacrifice this creature: Exile target noncreature artifact or noncreature enchantment.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenDies,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(2),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: "{G}, Sacrifice Haywire Mite: Exile target noncreature artifact
            // or noncreature enchantment."
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { green: 1, ..Default::default() }),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::ExileObject {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    non_creature: true,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
