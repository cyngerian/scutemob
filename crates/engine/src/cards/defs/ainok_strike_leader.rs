// Ainok Strike Leader — {1}{W}, Creature — Dog Warrior 2/2
// Whenever you attack with this creature and/or your commander, for each opponent, create
// a 1/1 red Goblin creature token that's tapped and attacking that player.
// Sacrifice this creature: Creature tokens you control gain indestructible until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ainok-strike-leader"),
        name: "Ainok Strike Leader".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: creature_types(&["Dog", "Warrior"]),
        oracle_text: "Whenever you attack with this creature and/or your commander, for each opponent, create a 1/1 red Goblin creature token that's tapped and attacking that player.\nSacrifice this creature: Creature tokens you control gain indestructible until end of turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Whenever you attack with this creature and/or your commander, for each
            // opponent, create a 1/1 red Goblin creature token that's tapped and attacking
            // that player."
            // DSL gaps:
            // 1. WhenAttacks trigger with "and/or your commander" condition not expressible.
            // 2. "For each opponent" ForEach loop over opponents creating a token attacking
            //    that specific opponent — entering tapped+attacking a specific player is
            //    not expressible in TokenSpec (enters_attacking is a bool, not per-player).
            //
            // TODO: "Sacrifice this creature: Creature tokens you control gain indestructible
            // until end of turn."
            // DSL gap: ApplyContinuousEffect with GrantKeyword(Indestructible) to
            // EffectFilter::CreatureTokensYouControl does not exist.
        ],
        ..Default::default()
    }
}
