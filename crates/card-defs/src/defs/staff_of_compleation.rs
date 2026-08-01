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
                // PB-DX4 (2026-08-01, OOS-DP10-8 triage): printed text is "Destroy target
                // permanent YOU OWN" (ownership, CR 108.3), authored here as
                // `TargetController::You` (control, CR 109.4). The two diverge in BOTH
                // directions under any control-change effect: a permanent you own but an
                // opponent controls (Mind Control) is wrongly an ILLEGAL target, and one you
                // control but do not own is wrongly a LEGAL one.
                //
                // `TargetFilter` has no owner axis at all, so this is not authorable today.
                // Deliberately NOT demoted, and NOT because the deviation is acceptable: this
                // is a corpus-wide approximation class (see `nether_traitor.rs`, whose own note
                // names `athreos` and `fecundity` as further instances), and demoting exactly
                // the members that happen to sit in PB-DP10's 97-def BASELINE would
                // misrepresent a class as a handful of cards. Filed as OOS-DX4-1: enumerate
                // every `Complete` def approximating an ownership clause with
                // `TargetController::You`, then decide the whole class at once.
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
        // PB-EF12 (EF-W-PB2-3): un-marked, see birds_of_paradise.rs for the fix.
        ..Default::default()
    }
}
