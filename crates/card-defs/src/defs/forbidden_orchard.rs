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
            },
            // CR 605.5a / CR 605.1b: "Whenever you tap this land for mana, target opponent
            // creates a 1/1 colorless Spirit creature token."
            // This trigger has a target, so it is NOT a mana ability (CR 605.5a) — it goes
            // on the stack normally. The trigger fires from the mana ability activation.
            //
            // PB-EF6 fixed BOTH target-side defects: (1) targets uses TargetOpponent (was
            // TargetPlayer, self-targetable); (2) TokenSpec.recipient routes the token to the
            // declared target opponent (PB-EF2; was defaulting to Controller, inverting the
            // card's drawback into an upside).
            //
            // STILL BLOCKED: Effect::AddManaAnyColor (the {T} mana ability above) always adds
            // Colorless instead of a chosen color — EF-W-PB2-3, still open. That is the sole
            // remaining reason this def is not Complete.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::This,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: OrdSet::new(),
                        card_types: [CardType::Creature].iter().copied().collect(),
                        subtypes: [SubType("Spirit".into())].iter().cloned().collect(),
                        count: EffectAmount::Fixed(1),
                        recipient: PlayerTarget::DeclaredTarget { index: 0 },
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
            "sole remaining blocker (EF-W-PB2-3): the {T} mana ability's Effect::AddManaAnyColor \
             always adds Colorless instead of a color the caster chooses — wrong game state on \
             the mana side. The target-side defects (self-targetable, token minted for the wrong \
             player) were fixed by PB-EF6 (TargetOpponent) + PB-EF2 (TokenSpec.recipient).",
        ),
        ..Default::default()
    }
}
