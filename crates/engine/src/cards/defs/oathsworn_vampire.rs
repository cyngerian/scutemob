// Oathsworn Vampire — {1}{B}, Creature — Knight Vampire 2/2
// This creature enters tapped.
// You may cast this card from your graveyard if you gained life this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("oathsworn-vampire"),
        name: "Oathsworn Vampire".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Knight", "Vampire"]),
        oracle_text: "This creature enters tapped.\nYou may cast this card from your graveyard if you gained life this turn.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this creature enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            // TODO: "You may cast this card from your graveyard if you gained life this turn."
            // DSL gap: graveyard casting permission with life-gained-this-turn condition.
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}
