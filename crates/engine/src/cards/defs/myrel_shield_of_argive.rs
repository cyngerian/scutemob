// Myrel, Shield of Argive — {3}{W}, Legendary Creature — Human Soldier 3/4
// During your turn, your opponents can't cast spells or activate abilities of artifacts,
// creatures, or enchantments.
// Whenever Myrel attacks, create X 1/1 colorless Soldier artifact creature tokens, where
// X is the number of Soldiers you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("myrel-shield-of-argive"),
        name: "Myrel, Shield of Argive".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "During your turn, your opponents can't cast spells or activate abilities of artifacts, creatures, or enchantments.\nWhenever Myrel attacks, create X 1/1 colorless Soldier artifact creature tokens, where X is the number of Soldiers you control.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: "Opponents can't cast/activate during your turn" stax restriction not in DSL.
            // Attack: create Soldier tokens equal to Soldiers you control
            // TODO: "X = number of Soldiers" — count-based EffectAmount not in DSL.
            //   Using fixed 2 as approximation.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        colors: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: 2,
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
        ],
        ..Default::default()
    }
}
