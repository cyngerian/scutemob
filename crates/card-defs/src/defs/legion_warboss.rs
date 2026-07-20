// Legion Warboss — {2}{R}, Creature — Goblin Soldier 2/2
// Mentor
// At the beginning of combat on your turn, create a 1/1 red Goblin creature token. That
// token gains haste until end of turn and attacks this combat if able.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("legion-warboss"),
        name: "Legion Warboss".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Soldier"]),
        oracle_text: "Mentor (Whenever this creature attacks, put a +1/+1 counter on target \
                      attacking creature with lesser power.)\nAt the beginning of combat on your \
                      turn, create a 1/1 red Goblin creature token. That token gains haste until \
                      end of turn and attacks this combat if able."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Mentor keyword not in DSL.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                // TODO: "Attacks this combat if able" forced-attack not in DSL.
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "TWO independent gaps, both live now that PB-RS3 closed the AtBeginningOfCombat sweep \
             gap (the token-creation trigger now actually fires). (1) Mentor keyword not in DSL \
             -- 0 matches for KeywordAbility::Mentor across the engine. (2) The created Goblin \
             token's 'and attacks this combat if able' is unimplemented -- the token can decline \
             to attack. Do NOT fix (2) by adding MustAttackEachCombat to TokenSpec.keywords: that \
             would apply EACH combat, permanently, over-restricting the token on every later \
             turn, whereas the oracle text scopes the requirement to the single combat it enters \
             attacking. Needs a same-combat-only forced-attack primitive (e.g. a delayed/one-shot \
             restriction keyed to the entering step), distinct from the evergreen \
             MustAttackEachCombat keyword.",
        ),
        ..Default::default()
    }
}
