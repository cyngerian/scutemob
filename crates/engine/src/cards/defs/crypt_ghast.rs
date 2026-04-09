// Crypt Ghast — {3}{B}, Creature — Spirit 2/2
// Extort
// Whenever you tap a Swamp for mana, add an additional {B}.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crypt-ghast"),
        name: "Crypt Ghast".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 1, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Extort (Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain that much life.)\nWhenever you tap a Swamp for mana, add an additional {B}.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Extort),
            // CR 605.1b / CR 106.12a: "Whenever you tap a Swamp for mana, add {B}."
            // Triggered mana ability — resolves immediately (CR 605.4a).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::LandSubtype(SubType("Swamp".into())),
                },
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: ManaPool { black: 1, ..Default::default() },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
