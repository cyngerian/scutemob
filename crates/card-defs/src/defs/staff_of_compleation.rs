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
                // PB-DX28 (closes OOS-DX4-1): printed text is "Destroy target permanent
                // YOU OWN" (CR 108.3 ownership), previously written here as
                // `TargetController::You` (CR 109.4 control) because `TargetFilter` had
                // no owner axis at all. The two diverge in BOTH directions under any
                // control-change effect: a permanent you own but an opponent controls
                // (Mind Control) was wrongly ILLEGAL, and one you control but do not own
                // was wrongly LEGAL. `TargetFilter.owner: TargetOwner` now exists
                // (enforced at `casting::validate_object_satisfies_requirement` and the
                // triggered-ability auto-target picker), so this is authored as the
                // printed clause directly. `controller: TargetController::You` is
                // REMOVED, not kept alongside — it was the stand-in, and keeping it
                // would make the card strictly narrower than printed (an owned-but-
                // opponent-controlled permanent must remain a legal target).
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    owner: TargetOwner::You,
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
        // PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs for the fix.
        ..Default::default()
    }
}
