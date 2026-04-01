// Thassa's Oracle — {U}{U}, Creature — Merfolk Wizard 1/3
// When this creature enters, look at the top X cards of your library, where X is your devotion
//   to blue. Put up to one of them on top of your library and the rest on the bottom of your
//   library in a random order. If X is greater than or equal to the number of cards in your
//   library, you win the game.
//
// TODO: "Look at top X cards where X = devotion to blue, put up to one on top, rest on bottom"
//   — no LookAtTopN-with-devotion-count + SelectAndRoute effect exists in DSL.
//   EffectAmount::DevotionTo(Color::Blue) exists but no Effect::LookAtTopN variant.
// TODO: "If X >= library size, you win the game" — no Effect::WinGame or condition that checks
//   devotion vs library count. DSL gap: need Effect::WinGame + Condition::DevotionGteLibrarySize.
//   Both halves of this ETB trigger are omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("thassas-oracle"),
        name: "Thassa's Oracle".to_string(),
        mana_cost: Some(ManaCost { blue: 2, ..Default::default() }),
        types: creature_types(&["Merfolk", "Wizard"]),
        oracle_text: "When this creature enters, look at the top X cards of your library, where X is your devotion to blue. Put up to one of them on top of your library and the rest on the bottom of your library in a random order. If X is greater than or equal to the number of cards in your library, you win the game. (Each {U} in the mana costs of permanents you control counts toward your devotion to blue.)".to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![
            // TODO: ETB trigger — look at top X (devotion to blue) cards, route one to top, rest
            //   to bottom. DSL gap: no Effect::LookAtTopN with EffectAmount::DevotionTo.
            // TODO: Win condition — if devotion >= library size, you win.
            //   DSL gap: no Effect::WinGame variant.
        ],
        ..Default::default()
    }
}
