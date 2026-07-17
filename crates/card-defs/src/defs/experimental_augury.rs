// Experimental Augury — {1}{U}, Instant
// Look at the top three cards of your library. Put one of them into your hand and
// the rest on the bottom of your library in any order. Proliferate.
//
// TODO: Interactive "choose 1 of 3" — M10 player choice. Approximated as
// DrawCards(1) + Proliferate.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("experimental-augury"),
        name: "Experimental Augury".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Look at the top three cards of your library. Put one of them into your hand \
                      and the rest on the bottom of your library in any order. Proliferate."
            .to_string(),
        abilities: vec![
            // TODO: Interactive top-3 selection deferred to M10.
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::Proliferate,
                ]),
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::known_wrong(
            "Oracle: 'Look at the top three cards of your library. Put one of them into your hand \
             and the rest on the bottom of your library in any order. Proliferate.' Def \
             approximates clause 1 as a blind DrawCards(1), which (a) skips the choice among \
             three and (b) leaves the other two on TOP instead of the BOTTOM — the library order \
             diverges permanently. Blocked on interactive selection (Effect::Choose is \
             non-interactive, effects/mod.rs:3190). Effect::RevealAndRoute could route top-3 by \
             filter but cannot express player choice. Proliferate is correct. Consider abilities: \
             vec![] per W5.",
        ),
        ..Default::default()
    }
}
