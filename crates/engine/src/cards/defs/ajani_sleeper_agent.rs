// Ajani, Sleeper Agent — {1}{G}{G/W/P}{W}, Legendary Planeswalker — Ajani
// Compleated ({G/W/P} can be paid with {G}, {W}, or 2 life. If life was paid, this
// planeswalker enters with two fewer loyalty counters.)
// +1: Reveal the top card of your library. If it's a creature or planeswalker card,
//     put it into your hand. Otherwise, you may put it on the bottom of your library.
// −3: Distribute three +1/+1 counters among up to three target creatures. They gain
//     vigilance until end of turn.
// −6: You get an emblem with "Whenever you cast a creature or planeswalker spell,
//     target opponent gets two poison counters."
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ajani-sleeper-agent"),
        name: "Ajani, Sleeper Agent".to_string(),
        // Mana cost: {1}{G}{G/W/P}{W}
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            white: 1,
            phyrexian: vec![PhyrexianMana::Hybrid(ManaColor::Green, ManaColor::White)],
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Ajani"],
        ),
        oracle_text: "Compleated ({G/W/P} can be paid with {G}, {W}, or 2 life. If life was paid, this planeswalker enters with two fewer loyalty counters.)\n+1: Reveal the top card of your library. If it's a creature or planeswalker card, put it into your hand. Otherwise, you may put it on the bottom of your library.\n\u{2212}3: Distribute three +1/+1 counters among up to three target creatures. They gain vigilance until end of turn.\n\u{2212}6: You get an emblem with \"Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.\"".to_string(),
        abilities: vec![
            // TODO: Compleated keyword — 2 fewer loyalty if Phyrexian life was paid.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                // +1: Reveal top card, put in hand if creature/planeswalker, else bottom of library.
                // TODO: RevealTop + conditional hand/library placement.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                // −3: Distribute three +1/+1 counters among up to three targets + vigilance.
                // TODO: distributed counter placement + grant vigilance until EOT.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(6),
                // −6: Create emblem with triggered ability.
                // TODO: Emblem creation (CR 114) is a known gap deferred to a dedicated
                // session. Tracked in docs/project-status.md Deferred Items table.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
            },
        ],
        starting_loyalty: Some(4),
        meld_pair: None,
        ..Default::default()
    }
}
