// Forbidden Orchard — Land
// {T}: Add one mana of any color.
// Whenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forbidden-orchard"),
        name: "Forbidden Orchard".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color.\nWhenever you tap this land for mana, \
                      target opponent creates a 1/1 colorless Spirit creature token."
            .to_string(),
        abilities: vec![
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 605.5a / CR 605.1b: "Whenever you tap this land for mana, target opponent
            // creates a 1/1 colorless Spirit creature token."
            // This trigger has a target, so it is NOT a mana ability (CR 605.5a) — it goes
            // on the stack normally. The trigger fires from the mana ability activation.
            //
            // PB-EF6 (2026-07-18): TargetRequirement::TargetOpponent now exists, and
            // TokenSpec.recipient (PB-EF2) can route the token to a declared target.
            // NOT wired here: `fire_mana_triggered_abilities` (rules/mana.rs) queues this
            // trigger as PendingTriggerKind::Normal with `ability_index` set to the RAW
            // index into `def.abilities`, but the flush_pending_triggers auto-target
            // picker for Normal-kind triggers reads `characteristics.triggered_abilities`
            // — which `enrich_spec_from_def` never populates for
            // `TriggerCondition::WhenTappedForMana` (unlike WhenEntersBattlefield /
            // WheneverCreatureDies / etc., which get a runtime TriggeredAbilityDef
            // conversion). So `targets` on THIS trigger is dead: the auto-picker always
            // sees an empty target list, and any effect referencing `DeclaredTarget{0}`
            // (a token recipient or otherwise) silently resolves to nothing. Proven
            // empirically: wiring `recipient: PlayerTarget::DeclaredTarget{index:0}` here
            // made `mana_triggers::test_mana_trigger_forbidden_orchard` create 0 Spirits
            // instead of the pre-existing (wrong-recipient) 1. New engine finding, not
            // fixed here (out of PB-EF6 scope — a mana.rs/enrich_spec_from_def dispatch
            // gap, orthogonal to both EF-W-PB2-2 and EF-W-PB2-3).
            //
            // TODO: token-for-target-opponent DSL gap. CreateToken creates tokens for the
            // controller; there is no working DSL path for "target player creates a
            // token" on a WhenTappedForMana trigger specifically (see above). As a
            // deterministic approximation, this creates the Spirit for the controller
            // (wrong beneficiary).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::This,
                },
                // Approximation: Spirit token for controller (should be for target opponent).
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: OrdSet::new(),
                        card_types: [CardType::Creature].iter().copied().collect(),
                        subtypes: [SubType("Spirit".into())].iter().cloned().collect(),
                        count: EffectAmount::Fixed(1),
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetOpponent],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::known_wrong(
            "PB-EF12 (EF-W-PB2-3) fixed the mana ability's colour-choice stub — it now resolves \
             to a real chosen colour instead of ManaColor::Colorless. Remaining blocker \
             (unrelated to colour, unfixed): oracle says 'target opponent creates a 1/1 colorless \
             Spirit'; the def creates the Spirit for Forbidden Orchard's OWN controller, \
             inverting the card's drawback into an upside. This is NOT a simple recipient-wiring \
             gap (PB-EF6 checked): the WhenTappedForMana trigger is queued by \
             rules/mana.rs::fire_mana_triggered_abilities as PendingTriggerKind::Normal with a \
             raw def.abilities index, but the auto-target picker for Normal-kind triggers reads \
             characteristics.triggered_abilities, which is never populated for WhenTappedForMana \
             by enrich_spec_from_def -- so this trigger's `targets` field is unreachable and \
             TokenSpec.recipient has nothing to read (proven: wiring it produced 0 tokens, not a \
             mis-targeted one). The WhenTappedForMana auto-target dispatch gap is a distinct, \
             unfiled engine finding.",
        ),
        ..Default::default()
    }
}
