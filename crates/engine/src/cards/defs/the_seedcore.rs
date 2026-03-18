// The Seedcore — Land — Sphere
// {T}: Add {C}.
// {T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.
// Corrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn.
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
                targets: vec![],
            },
            // {T}: Add one mana of any color. Spend this mana only to cast Phyrexian creature spells.
            // Note: "Phyrexian" is a creature type in MTG. Using CreatureWithSubtype to enforce
            // both the creature-spell and Phyrexian-subtype requirements per oracle text.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColorRestricted {
                    player: PlayerTarget::Controller,
                    restriction: ManaRestriction::CreatureWithSubtype(SubType(
                        "Phyrexian".to_string(),
                    )),
                },
                timing_restriction: None,
                targets: vec![],
            },
            // TODO: Corrupted — {T}: Target 1/1 creature gets +2/+1 until end of turn.
            // DSL gap: activated ability with targets + conditional activation
            // (opponent has 3+ poison counters) not expressible.
        ],
        ..Default::default()
    }
}
