// The World Tree
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-world-tree"),
        name: "The World Tree".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "This land enters tapped.\n{T}: Add {G}.\nAs long as you control six or more lands, lands you control have \"{T}: Add one mana of any color.\"\n{W}{W}{U}{U}{B}{B}{R}{R}{G}{G}, {T}, Sacrifice this land: Search your library for any number of God cards, put them onto the battlefield, then shuffle.".to_string(),
        abilities: vec![
            // CR 614.1c: self-replacement — this land enters tapped.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldEnterBattlefield {
                    filter: ObjectFilter::Any,
                },
                modification: ReplacementModification::EntersTapped,
                is_self: true,
                unless_condition: None,
            },
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 1, 0),
                },
                timing_restriction: None,
            },
            // TODO: Static — As long as you control six or more lands, lands you control have
            // "{T}: Add one mana of any color." DSL gap: count_threshold + grant-ability-to-permanents.
            // TODO: Activated — {W}{W}{U}{U}{B}{B}{R}{R}{G}{G}, {T}, Sacrifice: search library for
            // any number of God cards. DSL gap: multi-card search with type filter.
        ],
        ..Default::default()
    }
}
