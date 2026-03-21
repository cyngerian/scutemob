// Dryad Arbor — Land Creature — Forest Dryad (green via color indicator, CR 204)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dryad-arbor"),
        name: "Dryad Arbor".to_string(),
        mana_cost: None,
        color_indicator: Some(vec![Color::Green]),
        types: types_sub(&[CardType::Land, CardType::Creature], &["Forest", "Dryad"]),
        oracle_text: "(This land isn't a spell, it's affected by summoning sickness, and it has \"{T}: Add {G}.\")".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana { player: PlayerTarget::Controller, mana: mana_pool(0, 0, 0, 0, 1, 0) },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
            },
        ],
        ..Default::default()
    }
}
