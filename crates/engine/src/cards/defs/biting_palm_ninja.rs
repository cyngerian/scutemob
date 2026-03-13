// Biting-Palm Ninja — {2}{B}, Creature — Human Ninja 3/3
// Ninjutsu {2}{B}
// This creature enters with a menace counter on it.
// Whenever this creature deals combat damage to a player, you may remove a menace counter from it.
// When you do, that player reveals their hand and you choose a nonland card from it. Exile that card.
// TODO: DSL gap — "enters with a menace counter" requires CounterType::Menace, which doesn't exist.
// TODO: DSL gap — combat damage trigger with optional counter removal leading to a chained
// "when you do" trigger (hand reveal + exile chosen card) is beyond current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("biting-palm-ninja"),
        name: "Biting-Palm Ninja".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {2}{B}\nThis creature enters with a menace counter on it.\nWhenever this creature deals combat damage to a player, you may remove a menace counter from it. When you do, that player reveals their hand and you choose a nonland card from it. Exile that card.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 2, black: 1, ..Default::default() },
            },
            // TODO: ETB — enters with a menace counter.
            // DSL gap: CounterType::Menace doesn't exist.
            // TODO: triggered — combat damage to player → may remove menace counter → reveal hand, exile nonland card.
            // DSL gap: no "when you do" chained trigger; no hand-reveal + targeted exile from hand effect.
        ],
        ..Default::default()
    }
}
