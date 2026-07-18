// Rewind — {2}{U}{U}, Instant
// Counter target spell. Untap up to four lands.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rewind"),
        name: "Rewind".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Counter target spell. Untap up to four lands.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Counter the spell, then untap up to four lands.
            // Slot indexing (card_definition.rs:2799-2822 / effects/mod.rs resolve_effect_target_list
            // — ctx.targets[idx] reads the declared-target list at raw declaration-order
            // position idx, which validate_targets_inner's two-pass best-fit assignment
            // preserves): requirement slot 0 (TargetSpell, mandatory) consumes exactly one
            // declared-target position; requirement slot 1 (UpToN{4}) consumes the following
            // 0..4 positions. So index 0 = the countered spell, indices 1..4 = declared lands
            // (unfilled indices resolve to a no-op via resolve_effect_target_list returning
            // empty, CR 608.2b) — same "pooled indexing" convention already used across the
            // corpus (e.g. blessed_alliance.rs, untimely_malfunction.rs) for multi-slot target
            // lists, here combining one mandatory slot with a following UpToN slot.
            effect: Effect::Sequence(vec![
                Effect::CounterSpell {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    exile_instead: false,
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 1 },
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 2 },
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 3 },
                },
                Effect::UntapPermanent {
                    target: EffectTarget::DeclaredTarget { index: 4 },
                },
            ]),
            targets: vec![
                TargetRequirement::TargetSpell,
                TargetRequirement::UpToN {
                    count: 4,
                    inner: Box::new(TargetRequirement::TargetLand),
                },
            ],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
