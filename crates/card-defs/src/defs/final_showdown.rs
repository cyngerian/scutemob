// Final Showdown — {W}, Instant (Spree); choose one or more modes, each with an
// additional cost. Mode 0: all creatures lose all abilities until end of turn.
// Mode 1: a creature you control gains indestructible until end of turn.
// Mode 2 (+{3}{W}{W}): destroy all creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("final-showdown"),
        name: "Final Showdown".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Spree (Choose one or more additional costs.)\n+ {1} — All creatures lose \
                      all abilities until end of turn.\n+ {1} — Choose a creature you control. It \
                      gains indestructible until end of turn.\n+ {3}{W}{W} — Destroy all \
                      creatures."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Spree),
            AbilityDefinition::Spell {
                // CR 702.172a: Spree — at least one mode must be chosen; each chosen
                // mode's additional cost is paid on top of the card's base mana cost.
                effect: Effect::Sequence(vec![]),
                targets: vec![],
                modes: Some(ModeSelection {
                    // CR 702.172a: Spree requires at least 1 mode; no upper limit.
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    // CR 700.2h / 702.172a: per-mode additional costs.
                    // Mode 0: +{1}; Mode 1: +{1}; Mode 2: +{3}{W}{W}
                    mode_costs: Some(vec![
                        ManaCost {
                            generic: 1,
                            ..Default::default()
                        },
                        ManaCost {
                            generic: 1,
                            ..Default::default()
                        },
                        ManaCost {
                            generic: 3,
                            white: 2,
                            ..Default::default()
                        },
                    ]),
                    modes: vec![
                        // Mode 0 (+{1}): All creatures lose all abilities until end of turn.
                        // PB-AC7: unblocked — already-expressible via
                        // Effect::ApplyContinuousEffect { layer: Ability,
                        // modification: RemoveAllAbilities, filter: AllCreatures,
                        // duration: UntilEndOfTurn } (CR 613.1f).
                        Effect::ApplyContinuousEffect {
                            effect_def: Box::new(ContinuousEffectDef {
                                layer: EffectLayer::Ability,
                                modification: LayerModification::RemoveAllAbilities,
                                filter: EffectFilter::AllCreatures,
                                duration: EffectDuration::UntilEndOfTurn,
                                condition: None,
                            }),
                        },
                        // Mode 1 (+{1}): Choose a creature you control. It gains
                        // indestructible until end of turn.
                        // ENGINE-BLOCKED (still, per PB-AC7 roster review): the ruling
                        // clarifies this is a non-target selection made on resolution
                        // ("The second mode of Final Showdown doesn't target the
                        // creature. You don't choose which creature will gain
                        // indestructible until the spell is resolving."). The DSL has
                        // no "choose a permanent you control" resolution-time selection
                        // primitive (no EffectTarget::ChosenPermanentYouControl or
                        // equivalent) — only declared (cast-time) targets or fixed
                        // target sets (AllCreatures, etc.) exist. Using a real target
                        // would be CR-wrong (it must not be targeted); using AllCreatures
                        // you control would be wrong game state if you control more than
                        // one creature. This mode has no effect when resolved
                        // (no-op placeholder) — OOS-AC7-2.
                        Effect::Sequence(vec![]),
                        // Mode 2 (+{3}{W}{W}): Destroy all creatures (CR 701.8).
                        Effect::DestroyAll {
                            filter: TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                ..Default::default()
                            },
                            cant_be_regenerated: false,
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::partial(
            "(still, per PB-AC7 roster review): the ruling clarifies this is a non-target \
             selection made on resolution ('The second...",
        ),
        ..Default::default()
    }
}
