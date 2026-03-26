// Goblin Grenade — {R}, Sorcery
// As an additional cost to cast this spell, sacrifice a Goblin.
// Goblin Grenade deals 5 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-grenade"),
        name: "Goblin Grenade".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a Goblin.\nGoblin Grenade deals 5 damage to any target.".to_string(),
        // CR 118.8: Mandatory sacrifice of a Goblin as additional cost.
        spell_additional_costs: vec![SpellAdditionalCost::SacrificeSubtype(SubType("Goblin".to_string()))],
        abilities: vec![
            AbilityDefinition::Spell {
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(5),
                },
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
