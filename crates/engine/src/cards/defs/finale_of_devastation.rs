// Finale of Devastation — {X}{G}{G}, Sorcery
// Search your library and/or graveyard for a creature card with mana value X or less and
// put it onto the battlefield. If you search your library this way, shuffle. If X is 10
// or more, creatures you control get +X/+X and gain haste until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("finale-of-devastation"),
        name: "Finale of Devastation".to_string(),
        mana_cost: Some(ManaCost {
            green: 2,
            x_count: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Search your library and/or graveyard for a creature card with mana value X or less and put it onto the battlefield. If you search your library this way, shuffle. If X is 10 or more, creatures you control get +X/+X and gain haste until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                // CR 701.23: Search library and/or graveyard for a creature card.
                // TODO: max_cmc should be dynamic (XValue) — TargetFilter.max_cmc is Option<u32>,
                // not EffectAmount. Dynamic MV filter deferred until EffectAmount-based filter support
                // is added.
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: true,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
                // CR 107.3m: "If X is 10 or more, creatures you control get +X/+X and gain haste."
                // TODO: The +X/+X is dynamic (XValue), but LayerModification::ModifyBoth takes i32.
                // Approximated as +10/+10 when X >= 10 (the minimum valid X for this trigger).
                // Fix when LayerModification::ModifyBothDynamic(EffectAmount) is added.
                Effect::Conditional {
                    condition: Condition::XValueAtLeast(10),
                    if_true: Box::new(Effect::Sequence(vec![
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: crate::state::EffectLayer::PtModify,
                                modification: crate::state::LayerModification::ModifyBoth(10),
                                filter: crate::state::EffectFilter::CreaturesYouControl,
                                duration: crate::state::EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: crate::state::EffectLayer::Ability,
                                modification: crate::state::LayerModification::AddKeyword(
                                    KeywordAbility::Haste,
                                ),
                                filter: crate::state::EffectFilter::CreaturesYouControl,
                                duration: crate::state::EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                    ])),
                    if_false: Box::new(Effect::Nothing),
                },
            ]),
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
