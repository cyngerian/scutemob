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
        oracle_text: "{T}: Add one mana of any color.\nWhenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token.".to_string(),
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
            },
            // CR 605.5a / CR 605.1b: "Whenever you tap this land for mana, target opponent
            // creates a 1/1 colorless Spirit creature token."
            // This trigger has a target, so it is NOT a mana ability (CR 605.5a) — it goes
            // on the stack normally. The trigger fires from the mana ability activation.
            //
            // TODO: token-for-target-opponent DSL gap. CreateToken creates tokens for the
            // controller; there is no DSL for "target player creates a token". As a
            // deterministic approximation, this creates the Spirit for the controller (wrong
            // beneficiary). Correct implementation deferred to M10 when targeted player choice
            // is fully interactive. The trigger mechanism (WhenTappedForMana + stack queueing)
            // is correct.
            //
            // NOTE (LOW-4): TargetRequirement::TargetPlayer is used below, but oracle says
            // "target opponent". No TargetOpponent variant exists yet. Deferred until target
            // validation is fully implemented.
            AbilityDefinition::Triggered {
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
                        count: 1,
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetPlayer],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
