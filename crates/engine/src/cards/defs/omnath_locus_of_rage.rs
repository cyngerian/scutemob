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
            // Landfall — Whenever a land you control enters, create a 5/5 red and green
            // Elemental creature token. CR 207.2c: Landfall is an ability word.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elemental".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elemental".to_string())].into_iter().collect(),
                        colors: [Color::Red, Color::Green].into_iter().collect(),
                        power: 5,
                        toughness: 5,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // TODO: DSL gap — subtype-filtered death trigger (separate blocker from Landfall).
            // "Whenever Omnath or another Elemental you control dies, deal 3 damage to any target."
            // Requires TriggerCondition::WheneverCreatureDies with subtype filter + self-or-other
            // matching. This is independent of the Landfall ability above.
        ],
        ..Default::default()
    }
}
