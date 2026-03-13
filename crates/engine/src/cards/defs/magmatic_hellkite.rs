// Magmatic Hellkite — {2}{R}{R}, Creature — Dragon 4/5
// Flying
// When this creature enters, destroy target nonbasic land an opponent controls.
// Its controller searches their library for a basic land card, puts it onto the battlefield
// tapped with a stun counter on it, then shuffles.
// TODO: DSL gap — ETB trigger targeting a nonbasic land (non_land filter won't work — need
// nonbasic land specifically), opponent-controlled, then causing that controller to search
// for a basic land and put it into play tapped with a stun counter. Stun counters are not
// a CounterType variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("magmatic-hellkite"),
        name: "Magmatic Hellkite".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flying\nWhen this creature enters, destroy target nonbasic land an opponent controls. Its controller searches their library for a basic land card, puts it onto the battlefield tapped with a stun counter on it, then shuffles. (If a permanent with a stun counter would become untapped, remove one from it instead.)".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ETB — destroy target nonbasic land an opponent controls, then its controller
            // searches for a basic land and puts it into play tapped with a stun counter
        ],
        ..Default::default()
    }
}
