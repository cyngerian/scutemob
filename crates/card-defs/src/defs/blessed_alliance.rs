// Blessed Alliance — {1}{W} Instant, Escalate {2}, choose one or more
// Mode 0: Target player gains 4 life.
// Mode 1: Untap up to two target creatures. (approximated: untap DeclaredTarget 0 and 1)
// Mode 2: Target opponent sacrifices an attacking creature of their choice.
// PB-SFT: Mode 2 now uses is_attacking filter on SacrificePermanents.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blessed-alliance"),
        name: "Blessed Alliance".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Escalate {2} (Pay this cost for each mode chosen beyond the first.)\nChoose \
                      one or more —\n• Target player gains 4 life.\n• Untap up to two target \
                      creatures.\n• Target opponent sacrifices an attacking creature of their \
                      choice."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Escalate),
            AbilityDefinition::Escalate {
                cost: ManaCost {
                    generic: 2,
                    ..Default::default()
                },
            },
            AbilityDefinition::Spell {
                effect: Effect::Sequence(vec![]),
                // Targets are shared across all modes (flat, NOT migrated to
                // `mode_targets` — PB-AC4). Declared targets:
                //   index 0: TargetPlayer (mode 0 — gains 4 life)
                //   index 1: TargetCreature (mode 1 — first creature to untap)
                //   index 2: TargetCreature (mode 1 — second creature, "up to two")
                //   index 3: TargetPlayer (mode 2 — opponent who sacrifices)
                //
                // ENGINE-BLOCKED (two independent reasons, either alone would block this):
                // 1. This card has Escalate, and the engine HARD-REJECTS casting a spell
                //    that pays Escalate for 2+ modes when its `ModeSelection.mode_targets`
                //    is `Some` (`casting.rs`, PB-AC4 fix-phase Finding 1 — "Escalate
                //    combined with ModeSelection.mode_targets is not supported"). Migrating
                //    this card to `mode_targets` would make choosing more than one mode via
                //    Escalate uncastable, which is worse than the current flat-target
                //    approximation. Escalate's mandatory "choose one or more" +
                //    per-mode-targets combination is unsupported by design (see
                //    pb-plan-AC4.md scope boundary) — do not migrate.
                // 2. Independently, mode 1 ("up to two target creatures") is a
                //    variable-count target slot; the AC4 `mode_targets` design is
                //    fixed-count per mode and explicitly forbids nesting `UpToN` inside a
                //    `mode_targets[m]` entry (would make the flat-slice split ambiguous).
                targets: vec![
                    TargetRequirement::TargetPlayer,   // mode 0
                    TargetRequirement::TargetCreature, // mode 1 creature 1
                    TargetRequirement::TargetCreature, // mode 1 creature 2
                    TargetRequirement::TargetPlayer,   // mode 2 opponent
                ],
                modes: Some(ModeSelection {
                    min_modes: 1,
                    max_modes: 3,
                    allow_duplicate_modes: false,
                    mode_costs: None,
                    modes: vec![
                        // Mode 0: Target player gains 4 life.
                        Effect::GainLife {
                            player: PlayerTarget::DeclaredTarget { index: 0 },
                            amount: EffectAmount::Fixed(4),
                        },
                        // Mode 1: Untap up to two target creatures.
                        // TODO: "up to two" means the second target is optional; the DSL
                        // currently requires all declared targets to be legal. A future
                        // OptionalTarget or UpTo variant would handle this correctly.
                        Effect::Sequence(vec![
                            Effect::UntapPermanent {
                                target: EffectTarget::DeclaredTarget { index: 1 },
                            },
                            Effect::UntapPermanent {
                                target: EffectTarget::DeclaredTarget { index: 2 },
                            },
                        ]),
                        // Mode 2: Target opponent sacrifices an attacking creature of their choice.
                        // PB-SFT (CR 701.21a + CR 109.1): attacking creature filter.
                        // `is_attacking` is a runtime GameObject field checked explicitly at the
                        // SacrificePermanents resolution site (not in matches_filter).
                        Effect::SacrificePermanents {
                            player: PlayerTarget::DeclaredTarget { index: 3 },
                            count: EffectAmount::Fixed(1),
                            filter: Some(TargetFilter {
                                has_card_type: Some(CardType::Creature),
                                is_attacking: true,
                                ..Default::default()
                            }),
                        },
                    ],
                    mode_targets: None,
                }),
                cant_be_countered: false,
            },
        ],
        completeness: Completeness::partial(
            "(two independent reasons, either alone would block this): 1. This card has Escalate, \
             and the engine HARD-REJECTS...",
        ),
        ..Default::default()
    }
}
