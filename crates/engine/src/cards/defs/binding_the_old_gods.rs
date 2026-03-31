// Binding the Old Gods — {2}{B}{G}, Enchantment — Saga
// I — Destroy target nonland permanent an opponent controls.
// II — Search your library for a Forest card, put it onto the battlefield tapped, then shuffle.
// III — Creatures you control gain deathtouch until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("binding-the-old-gods"),
        name: "Binding the Old Gods".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Saga"]),
        oracle_text: "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\nI — Destroy target nonland permanent an opponent controls.\nII — Search your library for a Forest card, put it onto the battlefield tapped, then shuffle.\nIII — Creatures you control gain deathtouch until end of turn.".to_string(),
        abilities: vec![
            // Chapter I: Destroy target nonland permanent an opponent controls.
            AbilityDefinition::SagaChapter {
                chapter: 1,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    non_land: true,
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
            // Chapter II: Search library for a Forest card, put it onto battlefield tapped, shuffle.
            AbilityDefinition::SagaChapter {
                chapter: 2,
                effect: Effect::Sequence(vec![
                    Effect::SearchLibrary {
                        player: PlayerTarget::Controller,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Land),
                            has_subtype: Some(SubType("Forest".to_string())),
                            ..Default::default()
                        },
                        reveal: false,
                        destination: ZoneTarget::Battlefield { tapped: true },
                        shuffle_before_placing: false,
                        also_search_graveyard: false,
                    },
                    Effect::Shuffle { player: PlayerTarget::Controller },
                ]),
                targets: vec![],
            },
            // Chapter III: Creatures you control gain deathtouch until end of turn.
            AbilityDefinition::SagaChapter {
                chapter: 3,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::Ability,
                        modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                        filter: EffectFilter::CreaturesYouControl,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
