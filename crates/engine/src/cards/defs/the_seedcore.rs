// The Seedcore — Land — Sphere, {T}: Add {C}; {T}: Add any color (Phyrexian creatures only); Corrupted pump
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-seedcore"),
        name: "The Seedcore".to_string(),
        mana_cost: None,
        types: types_sub(&[CardType::Land], &["Sphere"]),
        oracle_text: "{T}: Add {C}.\n{T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.\nCorrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn. Activate only if an opponent has three or more poison counters.".to_string(),
        abilities: vec![
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
            },
            // {T}: Add one mana of any color.
            // TODO: Restriction "Spend this mana only to cast Phyrexian creature spells" not expressible.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
            },
            // TODO: Corrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn.
            // DSL gap: activated ability with targets (Activated has no targets field);
            // conditional activation (opponent has 3+ poison counters) not expressible.
        ],
        ..Default::default()
    }
}
