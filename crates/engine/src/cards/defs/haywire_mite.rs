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
            // TODO: DSL gap — "{G}, Sacrifice: Exile target noncreature artifact or
            // noncreature enchantment." Cost::Sequence([Mana, SacrificeSelf]) + target
            // filter for "noncreature artifact or noncreature enchantment" not in DSL.
        ],
        ..Default::default()
    }
}
