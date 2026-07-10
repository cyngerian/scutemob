// Vampire Gourmand — {1}{B}, Creature — Vampire 2/2
// Whenever this creature attacks, you may sacrifice another creature. If you do, draw a
// card and this creature can't be blocked this turn.
//
// ENGINE-BLOCKED: PB-AC2's Effect::MayPayThenEffect (CR 118.12) can express the "may
// sacrifice -> then draw + evasion" shape, and the "can't be blocked this turn" grant
// itself is expressible (KeywordAbility::CantBeBlocked via a temporary
// ApplyContinuousEffect). The blocker is `Cost::Sacrifice(filter)`: it has no
// "another" / exclude-self semantics (can_pay_optional_cost / pay_optional_cost only
// thread a PlayerId, not this permanent's own ObjectId), and Vampire Gourmand is
// itself a creature that would match a bare creature filter -- it could illegally
// sacrifice itself to trigger its own attack ability. Same gap documented on
// wight_of_the_reliquary.rs (Cost::SacrificeAnother does not exist). Omitted rather
// than risking the self-sacrifice edge case (W5 policy).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("vampire-gourmand"),
        name: "Vampire Gourmand".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Whenever this creature attacks, you may sacrifice another creature. If you do, draw a card and this creature can't be blocked this turn.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // ENGINE-BLOCKED: see module comment -- blocked on missing "another
            // creature" (exclude-self) sacrifice-cost semantics.
        ],
        completeness: Completeness::partial("PB-AC2's Effect::MayPayThenEffect (CR 118.12) can express the 'may sacrifice -> then draw + evasion' shape, and the..."),
        ..Default::default()
    }
}
