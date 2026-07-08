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
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverPermanentEntersBattlefield {
                    filter: Some(TargetFilter {
                        has_card_type: Some(CardType::Land),
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                    exclude_self: false,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Elemental".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Elemental".to_string())].into_iter().collect(),
                        colors: [Color::Red, Color::Green].into_iter().collect(),
                        power: 5,
                        toughness: 5,
                        count: EffectAmount::Fixed(1),
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
            // CR 603.10a: "Whenever Omnath or another Elemental you control dies."
            // exclude_self=false: fires when Omnath itself dies or another Elemental you control dies.
            // controller=You, filter=Elemental subtype (Omnath is an Elemental, so this covers it).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: false,
                    nontoken_only: false,
                    filter: Some(TargetFilter {
                        has_subtype: Some(SubType("Elemental".to_string())),
                        ..Default::default()
                    }),
                },
                effect: Effect::DealDamage {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(3),
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetAny],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
