// Witch Enchanter // Witch-Blessed Meadow — {3}{W} Creature — Human Warlock 2/2 // Land (MDFC)
// Oracle: "When this creature enters, destroy target artifact or enchantment an opponent controls."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("witch-enchanter"),
        name: "Witch Enchanter // Witch-Blessed Meadow".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warlock"]),
        oracle_text: "When this creature enters, destroy target artifact or enchantment an opponent controls.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 603.1: ETB trigger — destroy target artifact or enchantment an opponent controls.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
