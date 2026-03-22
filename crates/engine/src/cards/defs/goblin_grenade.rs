// Goblin Grenade — {R}, Sorcery
// As an additional cost to cast this spell, sacrifice a Goblin.
// Goblin Grenade deals 5 damage to any target.
//
// NOTE: "As an additional cost to cast this spell, sacrifice a Goblin" is a spell
// additional cost (CR 601.2b). The DSL has no required_additional_cost field on
// CardDefinition for spells with mandatory sacrifice-creature-type additional costs.
// The damage effect is implemented; the sacrifice enforcement is omitted per W5 policy.
// TODO: Add AdditionalCost::SacrificeSubtype(SubType) to the Spell DSL to model this.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-grenade"),
        name: "Goblin Grenade".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "As an additional cost to cast this spell, sacrifice a Goblin.\nGoblin Grenade deals 5 damage to any target.".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // TODO: Additional cost "sacrifice a Goblin" is not enforced — no
                // required_additional_cost field exists on CardDefinition for spell-cast
                // additional costs of specific creature subtypes. Omitted per W5 policy.
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
