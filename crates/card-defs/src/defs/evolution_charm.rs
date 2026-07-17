// Evolution Charm — {1}{G}, Instant
// Choose one —
// • Search your library for a basic land card, reveal it, put it into your hand, then shuffle.
// • Return target creature card from your graveyard to your hand.
// • Target creature gains flying until end of turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("evolution-charm"),
        name: "Evolution Charm".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Search your library for a basic land card, reveal it, put \
                      it into your hand, then shuffle.\n• Return target creature card from your \
                      graveyard to your hand.\n• Target creature gains flying until end of turn."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — mode 1 and mode 2 each declare
            // their own single target, LOCAL to that mode. `Spell.targets` is empty. Mode 0
            // has no targets (library search, self-referential).
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Search library for a basic land, reveal it, put into hand, shuffle.
                    Effect::Sequence(vec![
                        Effect::SearchLibrary {
                            player: PlayerTarget::Controller,
                            filter: basic_land_filter(),
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
                    // Mode 1: Return target creature card from your graveyard to your hand.
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand {
                            owner: PlayerTarget::Controller,
                        },
                        controller_override: None,
                    },
                    // Mode 2: Target creature gains flying until end of turn.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ],
                mode_targets: Some(vec![
                    vec![],
                    vec![TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    })],
                    vec![TargetRequirement::TargetCreature],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
