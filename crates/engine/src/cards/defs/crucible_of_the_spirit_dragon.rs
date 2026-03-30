// Crucible of the Spirit Dragon — Land, storage counter mechanics for Dragon mana (complex, abilities: vec![]).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crucible-of-the-spirit-dragon"),
        name: "Crucible of the Spirit Dragon".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add {C}.\n{1}, {T}: Put a storage counter on this land.\n{T}, Remove X storage counters from this land: Add X mana in any combination of colors. Spend this mana only to cast Dragon spells or activate abilities of Dragons.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // TODO: {1}, {T}: Put a storage counter on this land.
            // DSL gap: no Effect::AddCounter targeting self with CounterType::Storage.
            // TODO: {T}, Remove X storage counters: Add X mana in any combination of colors.
            // Spend this mana only to cast Dragon spells or activate abilities of Dragons.
            // DSL gap: X-removal cost + variable mana output + mana restriction not expressible.
        ],
        ..Default::default()
    }
}
