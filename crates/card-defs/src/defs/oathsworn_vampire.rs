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
            // CR 601.3: "You may cast this card from your graveyard if you gained life this turn."
            // Ruling 2018-01-19: Casting is permitted if ANY life was gained this turn,
            // regardless of subsequent life loss. The condition is checked at cast time.
            AbilityDefinition::CastSelfFromGraveyard {
                condition: Some(Box::new(Condition::ControllerGainedLifeThisTurn)),
                alt_mana_cost: None,
                additional_costs: vec![],
                required_alt_cost: None,
            },
        ],
        power: Some(2),
        toughness: Some(2),
        ..Default::default()
    }
}
