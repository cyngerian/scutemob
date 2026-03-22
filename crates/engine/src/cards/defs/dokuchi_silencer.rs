// Dokuchi Silencer — {1}{B}, Creature — Human Ninja 2/1
// Ninjutsu {1}{B}
// Whenever this creature deals combat damage to a player, you may discard a creature card.
// When you do, destroy target creature or planeswalker that player controls.
// TODO: DSL gap — "when you do" reflexive trigger: the combat damage trigger allows discarding
// a creature card, and only when that discard happens does a second trigger fire to destroy a
// creature or planeswalker that player controls. No DSL support for reflexive trigger chaining.
// Implementing: Ninjutsu keyword only.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dokuchi-silencer"),
        name: "Dokuchi Silencer".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Ninja"]),
        oracle_text: "Ninjutsu {1}{B} ({1}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)\nWhenever this creature deals combat damage to a player, you may discard a creature card. When you do, destroy target creature or planeswalker that player controls.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ninjutsu),
            AbilityDefinition::Ninjutsu {
                cost: ManaCost { generic: 1, black: 1, ..Default::default() },
            },
            // TODO: "Whenever this creature deals combat damage to a player, you may discard a
            // creature card. When you do, destroy target creature or planeswalker that player
            // controls." — requires reflexive trigger ("when you do") fired by a discard within
            // the combat damage trigger. DSL gap: no reflexive trigger support.
        ],
        ..Default::default()
    }
}
