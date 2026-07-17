// Staff of Compleation — {3} Artifact
// {T}, Pay 1 life: Destroy target permanent you own.
// {T}, Pay 2 life: Add one mana of any color.
// {T}, Pay 3 life: Proliferate.
// {T}, Pay 4 life: Draw a card.
// {5}: Untap this artifact.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("staff-of-compleation"),
        name: "Staff of Compleation".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}, Pay 1 life: Destroy target permanent you own.\n{T}, Pay 2 life: Add one mana of any color.\n{T}, Pay 3 life: Proliferate.\n{T}, Pay 4 life: Draw a card.\n{5}: Untap this artifact.".to_string(),
        abilities: vec![
            // {T}, Pay 1 life: Destroy target permanent you own.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(1),
                ]),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}, Pay 2 life: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(2),
                ]),
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}, Pay 3 life: Proliferate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(3),
                ]),
                effect: Effect::Proliferate,
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {T}, Pay 4 life: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Tap,
                    Cost::PayLife(4),
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // {5}: Untap Staff of Compleation.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 5, ..Default::default() }),
                effect: Effect::UntapPermanent { target: EffectTarget::Source },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        completeness: Completeness::known_wrong("Two independent defects, both probed. (1) CR 106.1b: '{T}, Pay 2 life: Add one mana of any color' adds one COLORLESS mana (life IS paid: probed 40 -> 38); colorless is not a color, so this is wrong state. (2) SF-9 — the OTHER three abilities pay NO life at all: flatten_cost_into (testing/replay_harness.rs) maps Cost::PayLife(_) => {} and ActivationCost has no life field, so '{T}, Pay 3 life: Proliferate' and '{T}, Pay 4 life: Draw a card' both probed at life 40 -> 40. This card ships a free proliferate and a free draw. See memory/card-authoring/sr34-engine-findings-2026-07-17.md SF-9."),
        ..Default::default()
    }
}
