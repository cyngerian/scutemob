// Stock Up — {2}{U}, Sorcery
// Look at the top five cards of your library. Put two of them into your hand
// and the rest on the bottom of your library in any order.
//
// CARDS-2 (scutemob-181) second fix cycle: oracle_text and the known_wrong note both
// previously said "bottom in a random order" — the printed card lets the player choose
// the order (i.e. worse than random for the engine: this is an unenforced-choice gap,
// not a randomization gap). Corrected both.
//
// TODO: DSL gap — "look at top N, choose M to put in hand, rest on bottom in any order"
// requires interactive player choice (select from top-5, then order the rest) which is
// deferred to M10 (Command::SelectLibraryCard). Approximated as DrawCards(2) + TODO note.
// "Bottom in any order" is also not expressible in current ZoneTarget variants.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("stock-up"),
        name: "Stock Up".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Look at the top five cards of your library. Put two of them into your hand \
                      and the rest on the bottom of your library in any order."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: "look at top 5, choose 2 to hand, rest on bottom" requires interactive
            // library-top selection (M10). Approximated as DrawCards(2) which draws from top
            // without the selection step. Upgrade when Command::SelectLibraryCard is available.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(2),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::known_wrong(
            "approximated as DrawCards(2). Deviates twice — no selection from the top five, and \
             the unchosen three stay on top instead of going to the bottom of the library in an \
             order the player chooses, so all later draws differ. Needs interactive top-N \
             selection (Command::SelectLibraryCard, M10) and a bottom-in-any-order ZoneTarget.",
        ),
        ..Default::default()
    }
}
