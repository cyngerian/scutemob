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
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Search your library for a basic land card, reveal it, put it into your hand, then shuffle.\n• Return target creature card from your graveyard to your hand.\n• Target creature gains flying until end of turn.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![]),
            // Targets:
            //   index 0: mode 1 — target creature card from your graveyard
            //   index 1: mode 2 — target creature (gains flying)
            // Mode 0 has no targets (library search, self-referential).
            targets: vec![
                TargetRequirement::TargetCardInYourGraveyard(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
                TargetRequirement::TargetCreature,
            ],
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
                            destination: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                            shuffle_before_placing: false,
                            also_search_graveyard: false,
                        },
                        Effect::Shuffle { player: PlayerTarget::Controller },
                    ]),
                    // Mode 1: Return target creature card from your graveyard to your hand.
                    Effect::MoveZone {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        to: ZoneTarget::Hand { owner: PlayerTarget::Controller },
                        controller_override: None,
                    },
                    // Mode 2: Target creature gains flying until end of turn.
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                            filter: EffectFilter::DeclaredTarget { index: 1 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ],
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
