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
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}, Pay 1 life: Destroy target permanent you own.\n{T}, Pay 2 life: Add \
                      one mana of any color.\n{T}, Pay 3 life: Proliferate.\n{T}, Pay 4 life: \
                      Draw a card.\n{5}: Untap this artifact."
            .to_string(),
        abilities: vec![
            // {T}, Pay 1 life: Destroy target permanent you own.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
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
                modes: None,
            },
            // {T}, Pay 2 life: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(2)]),
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {T}, Pay 3 life: Proliferate.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(3)]),
                effect: Effect::Proliferate,
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {T}, Pay 4 life: Draw a card.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(4)]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // {5}: Untap Staff of Compleation.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 5,
                    ..Default::default()
                }),
                effect: Effect::UntapPermanent {
                    target: EffectTarget::Source,
                },
                targets: vec![],
                timing_restriction: None,
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "CR 106.1b: '{T}, Pay 2 life: Add one mana of any color' adds one COLORLESS mana \
             (life IS paid correctly: probed 40 -> 38); colorless is not a color, so this is \
             wrong state (SF-11, memory/card-authoring/sr34-engine-findings-2026-07-17.md — \
             Effect::AddManaAnyColor produces ManaColor::Colorless on both the mana-ability and \
             stack paths). SF-9 (the OTHER three abilities paying no life at all) was fixed by \
             SR-36/scutemob-92: '{T}, Pay 3 life: Proliferate' now probes at life 40 -> 37 and \
             '{T}, Pay 4 life: Draw a card' at 40 -> 36 (see \
             tests/primitives/primitive_sr36_scaled_mana_and_life_costs.rs). The '{T}, Pay 1 \
             life: Destroy target permanent you own' ability also now pays (same fix, not \
             separately probed — it needs a target, same as before). Remaining blocker is the \
             colour bug above only.",
        ),
        ..Default::default()
    }
}
