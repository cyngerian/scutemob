// Reclamation Sage — {2}{G} Creature — Elf Shaman 2/1
// When this creature enters, you may destroy target artifact or enchantment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reclamation-sage"),
        name: "Reclamation Sage".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: creature_types(&["Elf", "Shaman"]),
        oracle_text: "When this creature enters, you may destroy target artifact or enchantment."
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // CR 603.1: ETB trigger — destroy target artifact or enchantment (optional).
            // "you may" is captured by targeting being optional at resolution.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
