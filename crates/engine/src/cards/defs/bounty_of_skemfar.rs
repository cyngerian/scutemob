// Bounty of Skemfar — {2}{G}, Sorcery
// Reveal the top six cards of your library. You may put up to one land card from
// among them onto the battlefield tapped and an Elf card into your hand. Put the
// rest on the bottom in random order.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bounty-of-skemfar"),
        name: "Bounty of Skemfar".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Reveal the top six cards of your library. You may put a land card from among them onto the battlefield tapped and an Elf card from among them into your hand. Put the rest on the bottom of your library in a random order.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // NOTE: Oracle says "up to one land + up to one Elf card"; DSL approximation routes
            // all matching lands to BF tapped and does not separately route Elf to hand.
            // Interactive "up to one" choice and dual-filter routing require M10 player choice
            // infrastructure. The land-to-BF portion is the higher-value effect.
            effect: Effect::RevealAndRoute {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(6),
                filter: TargetFilter {
                    has_card_type: Some(CardType::Land),
                    ..Default::default()
                },
                matched_dest: ZoneTarget::Battlefield { tapped: true },
                unmatched_dest: ZoneTarget::Library {
                    owner: PlayerTarget::Controller,
                    position: LibraryPosition::Bottom,
                },
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
