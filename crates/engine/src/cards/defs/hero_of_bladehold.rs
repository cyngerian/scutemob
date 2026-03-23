// Hero of Bladehold — {2}{W}{W}, Creature — Human Knight 3/4
// Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0
// until end of turn.)
// Whenever this creature attacks, create two 1/1 white Soldier creature tokens that
// are tapped and attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("hero-of-bladehold"),
        name: "Hero of Bladehold".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Battle cry (Whenever this creature attacks, each other attacking creature gets +1/+0 until end of turn.)\nWhenever this creature attacks, create two 1/1 white Soldier creature tokens that are tapped and attacking.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: Battle cry keyword not in DSL KeywordAbility enum.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: 2,
                        supertypes: im::OrdSet::new(),
                        keywords: im::OrdSet::new(),
                        tapped: true,
                        enters_attacking: true,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
