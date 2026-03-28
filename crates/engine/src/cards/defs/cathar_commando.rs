// Cathar Commando — {1}{W} Creature — Human Soldier 3/1
// Flash
// {1}, Sacrifice this creature: Destroy target artifact or enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cathar-commando"),
        name: "Cathar Commando".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Soldier"]),
        oracle_text:
            "Flash\n{1}, Sacrifice this creature: Destroy target artifact or enchantment."
                .to_string(),
        power: Some(3),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
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
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
