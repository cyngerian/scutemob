// Archmage's Charm — {U}{U}{U}, Instant
// Choose one —
// • Counter target spell.
// • Target player draws two cards.
// • Gain control of target nonland permanent with mana value 1 or less.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("archmages-charm"),
        name: "Archmage's Charm".to_string(),
        mana_cost: Some(ManaCost { blue: 3, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Counter target spell.\n• Target player draws two cards.\n• Gain control of target nonland permanent with mana value 1 or less.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // PB-AC4 (CR 700.2c/700.2f): per-mode targets — all three modes are fully
            // expressible (previously stubbed as `Effect::Nothing`; the plan flagged mode 2
            // "gain control + MV filter" as a likely blocker, but `Effect::GainControl` +
            // `TargetFilter.max_cmc` both already exist — verified against
            // `card_definition.rs`, not blocked). `Spell.targets` is empty; each mode's
            // target lives in `mode_targets` at LOCAL index 0.
            effect: Effect::Sequence(vec![]),
            targets: vec![],
            modes: Some(ModeSelection {
                min_modes: 1,
                max_modes: 1,
                allow_duplicate_modes: false,
                mode_costs: None,
                modes: vec![
                    // Mode 0: Counter target spell.
                    Effect::CounterSpell {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                    },
                    // Mode 1: Target player draws two cards.
                    Effect::DrawCards {
                        player: PlayerTarget::DeclaredTarget { index: 0 },
                        count: EffectAmount::Fixed(2),
                    },
                    // Mode 2: Gain control of target nonland permanent with mana value 1 or
                    // less. No stated duration -> indefinite (CR 613.1b default).
                    Effect::GainControl {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        duration: EffectDuration::Indefinite,
                    },
                ],
                mode_targets: Some(vec![
                    vec![TargetRequirement::TargetSpell],
                    vec![TargetRequirement::TargetPlayer],
                    vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                        non_land: true,
                        max_cmc: Some(1),
                        ..Default::default()
                    })],
                ]),
            }),
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
