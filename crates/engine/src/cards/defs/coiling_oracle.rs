// Coiling Oracle — {G}{U}, Creature — Snake Elf Druid 1/1
// When this enters, reveal the top card of your library. If it's a land card,
// put it onto the battlefield. Otherwise, put that card into your hand.
//
// TODO: DSL gap — "reveal top card, if land put onto battlefield, else put into hand" requires
// RevealAndRoute pattern: reveal top card of library, branch on card type (land → battlefield,
// non-land → hand). Neither Effect::RevealLibraryTop nor conditional branching by revealed
// card type exists in DSL. Approximated as Nothing to avoid wrong game state.
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
            // TODO: ETB — reveal top card of library; if land → battlefield, else → hand.
            // Needs Effect::RevealTopOfLibrary with branching on card type. DSL gap.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
