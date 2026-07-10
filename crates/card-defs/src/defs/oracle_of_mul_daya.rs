// Oracle of Mul Daya — {3}{G}, Creature — Elf Shaman 2/2
// You may play an additional land on each of your turns.
// Play with the top card of your library revealed.
// You may play lands from the top of your library.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oracle-of-mul-daya"),
        name: "Oracle of Mul Daya".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Creature], &["Elf", "Shaman"]),
        oracle_text: "You may play an additional land on each of your turns.\nPlay with the top card of your library revealed.\nYou may play lands from the top of your library.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 305.2 (PB-32): "You may play an additional land on each of your turns."
            AbilityDefinition::AdditionalLandPlays { count: 1 },
            // CR 601.3 / CR 305.1 (PB-A): "Play with the top card of your library revealed.
            // You may play lands from the top of your library."
            // reveal_top: true (all players see top card), LandsOnly filter.
            // 2021-03-19 ruling: land play from top counts against your land-play limit.
            AbilityDefinition::StaticPlayFromTop {
                filter: PlayFromTopFilter::LandsOnly,
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
