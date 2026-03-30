// Tetsuko Umezawa, Fugitive — {1}{U}, Legendary Creature — Human Rogue 1/3
// Creatures you control with power or toughness 1 or less can't be blocked.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tetsuko-umezawa-fugitive"),
        name: "Tetsuko Umezawa, Fugitive".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Rogue"],
        ),
        oracle_text: "Creatures you control with power or toughness 1 or less can't be blocked.".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: Static grant of CantBeBlocked to creatures you control with
            // power or toughness <= 1. Needs a power-or-toughness conditional
            // EffectFilter (existing filters check power only, not "power OR toughness").
        ],
        ..Default::default()
    }
}
