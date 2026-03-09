// Write into Being — {2}{U}, Sorcery; look at top 2, manifest one, put the
// other on top or bottom. CR 701.40.
//
// TODO: Full oracle text requires "look at top 2 cards, choose which to manifest,
// put the other on top or bottom" — interactive card selection is deferred to M10+
// (Command::SelectLibraryCard). Current implementation manifests the top card of
// the library directly, skipping the look-and-choose step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("write-into-being"),
        name: "Write into Being".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Look at the top two cards of your library. Manifest one of those cards, then put the other on the top or bottom of your library. (To manifest a card, put it onto the battlefield face down as a 2/2 creature. Turn it face up any time for its mana cost if it's a creature card.)".to_string(),
        abilities: vec![
            AbilityDefinition::Spell {
                // TODO: Should look at top 2 and let the player choose which to
                // manifest, then place the other on top or bottom. Interactive
                // library inspection deferred to M10+ (Command::SelectLibraryCard).
                effect: Effect::Manifest { player: PlayerTarget::Controller },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
