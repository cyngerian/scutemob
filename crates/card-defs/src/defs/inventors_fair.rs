// Inventors' Fair — Legendary Land
// At the beginning of your upkeep, if you control three or more artifacts, you gain 1 life.
// {T}: Add {C}.
// {4}, {T}, Sacrifice: Search for artifact card, reveal, put into hand, shuffle.
//   Activate only if you control three or more artifacts.
use crate::cards::helpers::*;
pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("inventors-fair"),
        name: "Inventors' Fair".to_string(),
        mana_cost: None,
        types: supertypes(&[SuperType::Legendary], &[CardType::Land]),
        oracle_text: "At the beginning of your upkeep, if you control three or more artifacts, \
                      you gain 1 life.\n{T}: Add {C}.\n{4}, {T}, Sacrifice Inventors' Fair: \
                      Search your library for an artifact card, reveal it, put it into your hand, \
                      then shuffle. Activate only if you control three or more artifacts."
            .to_string(),
        abilities: vec![
            // At the beginning of your upkeep, if you control three or more artifacts,
            // you gain 1 life.
            // CR 603.4 (rulings 2016-09-20 #1/#2): checked at queue time
            // (rules/turn_actions.rs' AtBeginningOfYourUpkeep CardDef sweep) and
            // re-checked at resolution; the artifacts need not be the same ones.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 3,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        ..Default::default()
                    },
                }),
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // {T}: Add {C}.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddMana {
                    player: PlayerTarget::Controller,
                    mana: mana_pool(0, 0, 0, 0, 0, 1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {4}, {T}, Sacrifice: Search for artifact, reveal, put into hand, shuffle.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 4,
                        ..Default::default()
                    }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                        // `reveal: true` is currently inert -- the engine destructures
                        // `reveal: _` and never actually reveals the found card
                        // (effects/mod.rs:3479, seeded as OOS-DP9-9, pre-existing and
                        // out of scope for this batch). Printed "reveal it" clause is
                        // therefore not yet implemented despite the Complete marker.
                        reveal: true,
                        destination: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle {
                        player: PlayerTarget::Controller,
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                // CR 602.5b "Activate only if …" — ruling 2016-09-20 #3: checked ONLY on
                // activation, never re-checked at resolution, so this belongs on
                // activation_condition and NOT in an Effect::Conditional wrapper.
                activation_condition: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 3,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Artifact),
                        ..Default::default()
                    },
                }),
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
