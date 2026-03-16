// Adrix and Nev, Twincasters — {2}{G}{U}, Legendary Creature — Merfolk Wizard 2/2
// Ward {2}
// If one or more tokens would be created under your control, twice that many of
// those tokens are created instead.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("adrix-and-nev-twincasters"),
        name: "Adrix and Nev, Twincasters".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Merfolk", "Wizard"]),
        oracle_text:
            "Ward {2} (Whenever this creature becomes the target of a spell or ability an opponent controls, counter it unless that player pays {2}.)\n\
             If one or more tokens would be created under your control, twice that many of those tokens are created instead."
                .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // CR 111.1 / CR 614.1: Token-doubling replacement effect.
            // PlayerId(0) placeholder — bound to controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldCreateTokens {
                    controller_filter: PlayerFilter::Specific(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleTokens,
                is_self: false,
                unless_condition: None,
            },
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        meld_pair: None,
    }
}
