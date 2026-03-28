// Fellwar Stone — {2} Artifact; {T}: Add one mana of any color that a land
// an opponent controls could produce.
// TODO: DSL gap — same as Exotic Orchard. Simplified to any color.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("fellwar-stone"),
        name: "Fellwar Stone".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color that a land an opponent controls could produce.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Tap,
            effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
            timing_restriction: None,
            targets: vec![],
                activation_condition: None,
                activation_zone: None,
        }],
        ..Default::default()
    }
}
