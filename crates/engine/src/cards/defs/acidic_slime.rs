// Acidic Slime — {3}{G}{G} Creature — Ooze 2/2
// Deathtouch
// When this creature enters, destroy target artifact, enchantment, or land.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("acidic-slime"),
        name: "Acidic Slime".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 2, ..Default::default() }),
        types: creature_types(&["Ooze"]),
        oracle_text:
            "Deathtouch (Any amount of damage this deals to a creature is enough to destroy it.)\nWhen this creature enters, destroy target artifact, enchantment, or land."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // CR 603.1: ETB trigger — destroy target artifact, enchantment, or land.
            // Three card types with OR semantics via has_card_types.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![
                        CardType::Artifact,
                        CardType::Enchantment,
                        CardType::Land,
                    ],
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
