// Street Wraith — {3}{B}{B}, Creature — Wraith 3/4
// Swampwalk
// Cycling—Pay 2 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("street-wraith"),
        name: "Street Wraith".to_string(),
        mana_cost: Some(ManaCost { generic: 3, black: 2, ..Default::default() }),
        types: creature_types(&["Wraith"]),
        oracle_text: "Swampwalk (This creature can't be blocked as long as defending player controls a Swamp.)\nCycling\u{2014}Pay 2 life. (Pay 2 life, Discard this card: Draw a card.)".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Landwalk(
                LandwalkType::BasicType(SubType("Swamp".to_string())),
            )),
            // TODO: Cycling with life cost (pay 2 life) — Cycling DSL only accepts ManaCost.
            //   Needs LifeCycling DSL extension. Stripped to avoid free cycling.
        ],
        ..Default::default()
    }
}
