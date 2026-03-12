// Skemfar Elderhall
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skemfar-elderhall"),
        name: "Skemfar Elderhall".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\n{2}{B}{B}{G}, {T}, Sacrifice this land: Up to one target creature you don't control gets -2/-2 until end of turn. Create two 1/1 green Elf Warrior creature tokens. Activate only as a sorcery.".to_string(),
        abilities: vec![
            // TODO: This land enters tapped.
            // TODO: Activated — {T}: Add {G}.
            // TODO: Activated — {2}{B}{B}{G}, {T}, Sacrifice this land: Up to one target creature you don't cont
        ],
        ..Default::default()
    }
}
