// Falkenrath Pit Fighter — {R}, Creature — Vampire Warrior 2/1
// {1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only if an
// opponent lost life this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("falkenrath-pit-fighter"),
        name: "Falkenrath Pit Fighter".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Vampire", "Warrior"]),
        oracle_text: "{1}{R}, Discard a card, Sacrifice a Vampire: Draw two cards. Activate only \
                      if an opponent lost life this turn."
            .to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            // TODO: "Sacrifice a Vampire" (not self) + "opponent lost life this turn"
            //   activation condition not expressible. SacrificeSelf was wrong (oracle
            //   says sacrifice any Vampire, not specifically self). Removed to avoid
            //   wrong game state.
        ],
        completeness: Completeness::partial(
            "Blocked on activation condition 'only if an opponent lost life this turn' — no \
             Condition::OpponentLostLifeThisTurn (the tracking exists for AltCostKind::Spectacle \
             but is not exposed as a Condition). The cost itself ({1}{R}, Cost::DiscardCard, \
             Cost::Sacrifice(Vampire filter) via Cost::Sequence) is expressible today.",
        ),
        ..Default::default()
    }
}
