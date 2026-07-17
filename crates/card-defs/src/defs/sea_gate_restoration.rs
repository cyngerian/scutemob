// Sea Gate Restoration // Sea Gate, Reborn — Draw cards equal to the number of cards in your hand plus one. You hav
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sea-gate-restoration"),
        name: "Sea Gate Restoration // Sea Gate, Reborn".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            blue: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Draw cards equal to the number of cards in your hand plus one. You have no \
                      maximum hand size for the rest of the game."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Front face is unblocked as of PB-AC9 — author as Effect::Sequence([DrawCards { \
             count: EffectAmount::Sum(HandSize + 1) or equivalent }, SetNoMaximumHandSize { \
             player: Controller }]); note the draw count must be locked in before the draws begin \
             (CR 608.2h). Remaining work: MDFC back face Sea Gate, Reborn (Land, 'enters tapped \
             unless you pay 3 life') via back_face + \
             ReplacementModification::EntersTappedUnlessPayLife.",
        ),
        ..Default::default()
    }
}
