// Tatyova, Steward of Tides — {G}{G}{U}, Legendary Creature — Merfolk Druid 3/3
// Land creatures you control have flying; Landfall (7+ lands): animate target land 3/3 Elemental haste
// TODO: grant flying to land-creatures (continuous effect with card type filter) and landfall animate-land not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tatyova-steward-of-tides"),
        name: "Tatyova, Steward of Tides".to_string(),
        mana_cost: Some(ManaCost {
            green: 2,
            blue: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Merfolk", "Druid"],
        ),
        oracle_text: "Land creatures you control have flying.\nWhenever a land you control enters, if you control seven or more lands, up to one target land you control becomes a 3/3 Elemental creature with haste. It's still a land.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // TODO: Continuous effect granting flying to land creatures requires a filter on
            // card types (Land + Creature) which is not expressible as an EffectFilter.
            // TODO: Landfall trigger with count_threshold condition (7+ lands) that animates
            // a targeted land as 3/3 Elemental with haste — requires targeted_trigger with
            // intervening-if count check and animate-land effect, none of which is in DSL.
        ],
        ..Default::default()
    }
}
