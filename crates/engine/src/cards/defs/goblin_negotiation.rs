// Goblin Negotiation — {X}{R}{R}, Sorcery
// Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin
// creature tokens equal to the amount of excess damage dealt to that creature this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-negotiation"),
        name: "Goblin Negotiation".to_string(),
        mana_cost: Some(ManaCost { red: 2, x_count: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Goblin Negotiation deals X damage to target creature. Create a number of 1/1 red Goblin creature tokens equal to the amount of excess damage dealt to that creature this way.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 107.3m: Deal X damage to target creature.
            effect: Effect::Sequence(vec![
                Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::XValue,
                },
                // TODO: "Create tokens equal to excess damage" — requires tracking damage
                // dealt minus toughness of the target. The DSL has no way to compute
                // excess damage at effect resolution time. Deferred.
            ]),
            targets: vec![TargetRequirement::TargetCreature],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
