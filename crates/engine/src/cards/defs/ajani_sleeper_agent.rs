// Ajani, Sleeper Agent — {1}{G}{G/W/P}{W}, Legendary Planeswalker — Ajani
// Compleated
// TODO: DSL gap — Planeswalker type not represented in CardType enum; loyalty abilities not
//   supported in card DSL. All three loyalty abilities (+1/-3/-6) require complex effects:
//   +1: reveal top card, put in hand if creature/planeswalker
//   -3: distribute +1/+1 counters among up to 3 targets + grant vigilance until EOT
//   -6: create emblem with triggered poison counter ability
// TODO: DSL gap — Compleated hybrid mana ({G/W/P}) not representable in ManaCost struct
//   (no life-payment mana field)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ajani-sleeper-agent"),
        name: "Ajani, Sleeper Agent".to_string(),
        // Mana cost approximated as {1}{G}{W} — the {G/W/P} hybrid pip omits life payment
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Ajani"],
        ),
        oracle_text: "Compleated ({G/W/P} can be paid with {G}, {W}, or 2 life. If life was paid, this planeswalker enters with two fewer loyalty counters.)\n+1: Reveal the top card of your library. If it's a creature or planeswalker card, put it into your hand. Otherwise, you may put it on the bottom of your library.\n−3: Distribute three +1/+1 counters among up to three target creatures. They gain vigilance until end of turn.\n−6: You get an emblem with \"Whenever you cast a creature or planeswalker spell, target opponent gets two poison counters.\"".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
