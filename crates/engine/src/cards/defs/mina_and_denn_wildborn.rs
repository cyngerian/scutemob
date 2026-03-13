// Mina and Denn, Wildborn — {2}{R}{G}, Legendary Creature — Elf Ally 4/4
// You may play an additional land on each of your turns.
// {R}{G}, Return a land you control to its owner's hand: Target creature gains trample until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mina-and-denn-wildborn"),
        name: "Mina and Denn, Wildborn".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elf", "Ally"],
        ),
        oracle_text: "You may play an additional land on each of your turns.\n{R}{G}, Return a land you control to its owner's hand: Target creature gains trample until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // TODO: Static ability — you may play an additional land on each of your turns.
            // DSL gap: no additional-land-drop static effect.
            // TODO: Activated ability — {R}{G}, return a land you control to its owner's hand:
            // target creature gains trample until end of turn.
            // DSL gap: no return-land cost; no targeted grant-keyword until-end-of-turn effect on activated abilities.
        ],
        ..Default::default()
    }
}
