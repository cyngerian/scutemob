// Omnath, Locus of Rage — {3}{R}{R}{G}{G} Legendary Creature — Elemental 5/5
// Landfall — Whenever a land you control enters, create a 5/5 red and green Elemental token.
// Whenever Omnath or another Elemental you control dies, Omnath deals 3 damage to any target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("omnath-locus-of-rage"),
        name: "Omnath, Locus of Rage".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, green: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elemental"],
        ),
        oracle_text: "Landfall \u{2014} Whenever a land you control enters, create a 5/5 red and green Elemental creature token.\nWhenever Omnath, Locus of Rage or another Elemental you control dies, Omnath deals 3 damage to any target.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // Landfall — create a 5/5 red and green Elemental creature token.
            // TODO: TriggerCondition::Landfall (WhenALandYouControlEnters) not in DSL.
            // Whenever ~ or another Elemental you control dies, deal 3 damage.
            // TODO: Subtype-filtered death trigger not in DSL.
        ],
        ..Default::default()
    }
}
