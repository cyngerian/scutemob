// Gemstone Caverns — Legendary Land, conditional ETB replacement + luck counter mechanic
// TODO: "If this card is in your opening hand and you're not the starting player" ETB
// replacement — opening hand / starting player check not expressible in DSL.
// TODO: "{T}: Add {C}. If Gemstone Caverns has a luck counter on it, instead add one mana
// of any color." — conditional mana based on counter state not expressible in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gemstone-caverns"),
        name: "Gemstone Caverns".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "If this card is in your opening hand and you're not the starting player, \
                      you may begin the game with Gemstone Caverns on the battlefield with a luck \
                      counter on it. If you do, exile a card from your hand.\n{T}: Add {C}. If \
                      Gemstone Caverns has a luck counter on it, instead add one mana of any \
                      color."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Blocked on a conditional opening-hand start: AbilityDefinition::OpeningHand is an \
             unconditional marker (start_game moves the card to the battlefield with no condition \
             and no payload). Needs a 'not the starting player' gate, an \
             enters-with-a-luck-counter payload, and an 'if you do, exile a card from your hand' \
             cost. The mana ability is NOT a blocker — Effect::Conditional + \
             Condition::SourceHasCounters(Luck) + AddManaAnyColor expresses it today.",
        ),
        ..Default::default()
    }
}
