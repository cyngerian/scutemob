// Future Sight — {2}{U}{U}{U}, Enchantment
// Play with the top card of your library revealed. You may play lands and cast spells
// from the top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("future-sight"),
        name: "Future Sight".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 3, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Play with the top card of your library revealed. You may play lands and cast spells from the top of your library.".to_string(),
        abilities: vec![
            // CR 601.3 / CR 305.1 (PB-A): "Play with the top card of your library revealed.
            // You may play lands and cast spells from the top of your library."
            // All filter: covers all card types (lands, creatures, spells, etc.).
            // reveal_top: true means ALL players see the top card (oracle text: "revealed").
            // 2019-06-14 ruling: Normal timing restrictions still apply.
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::All,
                look_at_top: false,
                reveal_top: true,
                pay_life_instead: false,
                condition: None,
                on_cast_effect: None,
            },
        ],
        ..Default::default()
    }
}
