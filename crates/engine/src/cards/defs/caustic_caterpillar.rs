// Caustic Caterpillar — {G}, Creature — Insect 1/1
// {1}{G}, Sacrifice this creature: Destroy target artifact or enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("caustic-caterpillar"),
        name: "Caustic Caterpillar".to_string(),
        mana_cost: Some(ManaCost { green: 1, ..Default::default() }),
        types: creature_types(&["Insect"]),
        oracle_text: "{1}{G}, Sacrifice this creature: Destroy target artifact or enchantment.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // {1}{G}, Sacrifice this creature: Destroy target artifact or enchantment.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, green: 1, ..Default::default() }),
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    ..Default::default()
                })],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
