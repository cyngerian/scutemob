// Krenko, Mob Boss — {2}{R}{R}, Legendary Creature — Goblin Warrior 3/3
// {T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("krenko-mob-boss"),
        name: "Krenko, Mob Boss".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Warrior"],
        ),
        oracle_text: "{T}: Create X 1/1 red Goblin creature tokens, where X is the number of \
                      Goblins you control."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // {T}: Create X 1/1 red Goblin creature tokens, where X is the number of Goblins you control.
            // CR 111.1 / CR 608.2h: PermanentCount resolved at activation resolution time.
            // Krenko himself counts as a Goblin (he's a Goblin Warrior on the battlefield).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        supertypes: imbl::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        keywords: imbl::OrdSet::new(),
                        count: EffectAmount::PermanentCount {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                has_subtype: Some(SubType("Goblin".to_string())),
                                controller: TargetController::You,
                                ..Default::default()
                            },
                            controller: PlayerTarget::Controller,
                        },
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        ..Default::default()
    }
}
