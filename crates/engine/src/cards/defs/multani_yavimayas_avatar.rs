// Multani, Yavimaya's Avatar — {4}{G}{G}, Legendary Creature — Elemental Avatar 0/0
// Reach, trample; P/T = number of lands you control + lands in graveyard
// TODO: dynamic P/T based on land count (count_threshold gap); graveyard-return activated ability
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("multani-yavimayas-avatar"),
        name: "Multani, Yavimaya's Avatar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            green: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental", "Avatar"],
        ),
        oracle_text: "Reach, trample\nMultani gets +1/+1 for each land you control and each land card in your graveyard.\n{1}{G}, Return two lands you control to their owner's hand: Return this card from your graveyard to your hand.".to_string(),
        power: None,   // */* CDA — engine SBA skips None toughness
        toughness: None,
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Reach),
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: +1/+1 for each land you control and each land in graveyard requires
            // dynamic count-based P/T modification not supported in current DSL.
            // TODO: Activated ability to return from graveyard requires return_from_graveyard
            // and a cost of returning lands, neither of which is in the DSL.
        ],
        ..Default::default()
    }
}
