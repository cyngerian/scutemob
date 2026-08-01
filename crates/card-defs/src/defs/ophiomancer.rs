// Ophiomancer — {2}{B}, Creature — Human Shaman 2/2
// At the beginning of each upkeep, if you control no Snakes, create a 1/1 black Snake
// creature token with deathtouch.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ophiomancer"),
        name: "Ophiomancer".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Shaman"]),
        oracle_text: "At the beginning of each upkeep, if you control no Snakes, create a 1/1 \
                      black Snake creature token with deathtouch."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 603.4 (rulings 2013-10-17 #1/#2): "if you control no Snakes" is checked
            // BOTH at queue time (rules/turn_actions.rs's AtBeginningOf{Your,Each}Upkeep
            // CardDef sweep, PB-DP6) and re-checked at resolution
            // (InterveningIf::CardDef, PB-DX1).
            //
            // PB-DX3b: the def's own former note was right that the DSL gap was stale, but
            // wrong about which variant to use. `Condition::ControlCreatureWithSubtype`
            // hard-requires CardType::Creature (effects/mod.rs) — CR reads "you control no
            // **Snakes**" (permanents with the Snake subtype), a superset in principle,
            // though 2013-era Snakes were all creatures so the ruling text and this
            // TargetFilter agree in practice. `has_subtype` alone, with no
            // `has_card_type` restriction, is the precise translation.
            //
            // `AtBeginningOfEachUpkeep` fires on every player's upkeep, but "if YOU control
            // no Snakes" gates against Ophiomancer's *controller*
            // (check_static_condition's `ctx.controller`), not the active player whose
            // upkeep it is — pinned by T7.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfEachUpkeep,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Snake".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Snake".to_string())].into_iter().collect(),
                        colors: [Color::Black].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Deathtouch].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: Some(Condition::Not(Box::new(
                    Condition::YouControlNOrMoreWithFilter {
                        count: 1,
                        filter: TargetFilter {
                            has_subtype: Some(SubType("Snake".to_string())),
                            ..Default::default()
                        },
                    },
                ))),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
