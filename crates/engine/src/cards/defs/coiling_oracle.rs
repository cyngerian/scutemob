// Coiling Oracle — {G}{U}, Creature — Snake Elf Druid 1/1
// When this enters, reveal the top card of your library. If it's a land card,
// put it onto the battlefield. Otherwise, put that card into your hand.
//
// CR 701.20: "To reveal a card, show that card to all players for a brief time."
// Effect::RevealAndRoute added in PB-22 with ZoneTarget::Battlefield support.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("coiling-oracle"),
        name: "Coiling Oracle".to_string(),
        mana_cost: Some(ManaCost { green: 1, blue: 1, ..Default::default() }),
        types: creature_types(&["Snake", "Elf", "Druid"]),
        oracle_text: "When this enters, reveal the top card of your library. If it's a land card, put it onto the battlefield. Otherwise, put that card into your hand.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 701.20: ETB — reveal top card of library; if land → battlefield (untapped),
            // else → hand. Uses RevealAndRoute with CardType::Land filter.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::RevealAndRoute {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    matched_dest: ZoneTarget::Battlefield { tapped: false },
                    unmatched_dest: ZoneTarget::Hand {
                        owner: PlayerTarget::Controller,
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
