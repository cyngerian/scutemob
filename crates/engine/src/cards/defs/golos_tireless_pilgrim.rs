// Golos, Tireless Pilgrim — {5} Legendary Artifact Creature — Scout 3/5
// ETB: search library for a land, put it onto battlefield tapped, shuffle.
// {2}{W}{U}{B}{R}{G}: Exile top 3 cards of library. You may play them this turn without paying their mana costs.
//
// DSL gap: the 5-color activated ability requires PlayExiledCard and exile-top-N effects
//   chained together; no Effect::ExileTopN + allow free play combination in DSL.
// ETB search ability is faithfully implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("golos-tireless-pilgrim"),
        name: "Golos, Tireless Pilgrim".to_string(),
        mana_cost: Some(ManaCost { generic: 5, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact, CardType::Creature], &["Scout"]),
        oracle_text: "When Golos enters, you may search your library for a land card, put that card onto the battlefield tapped, then shuffle.\n{2}{W}{U}{B}{R}{G}: Exile the top three cards of your library. You may play them this turn without paying their mana costs.".to_string(),
        power: Some(3),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Land),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: true },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                intervening_if: None,
                targets: vec![],
            },
            // TODO: {2}{W}{U}{B}{R}{G}: Exile top 3 cards, you may play them this turn without paying mana costs
            //   (no Effect::ExileTopCards + free-play-until-end-of-turn combination in DSL)
        ],
        ..Default::default()
    }
}
