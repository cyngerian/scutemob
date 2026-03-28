// Chromatic Lantern — {3} Artifact; "Lands you control have {T}: Add any color" (static grant);
// {T}: Add one mana of any color.
// TODO: "Lands you control have '{T}: Add one mana of any color.'" is a DSL gap —
// no GrantActivatedAbility static effect primitive exists.
// The self tap-for-any-color ability is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chromatic-lantern"),
        name: "Chromatic Lantern".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Lands you control have \"{T}: Add one mana of any color.\"\n{T}: Add one mana of any color.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
