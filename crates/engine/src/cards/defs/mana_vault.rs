// Mana Vault — {1} Artifact
// This artifact doesn't untap during your untap step.
// At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact.
// At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.
// {T}: Add {C}{C}{C}.
//
// ENGINE-BLOCKED: "At the beginning of your draw step, if this artifact is tapped, it deals
//   1 damage to you." — there is no TriggerCondition::AtBeginningOfYourDrawStep (verified:
//   TriggerCondition only has AtBeginningOfYourUpkeep/EachUpkeep/YourEndStep/Combat — no
//   draw-step trigger exists anywhere in the enum). Genuinely missing.
// ENGINE-BLOCKED: "{T}: Add {C}{C}{C}." — per W5 policy (KI-13 class), this mana ability's
//   real cost/risk is inseparable from the draw-step damage-if-tapped clause above: once
//   tapped, Mana Vault stays tapped until the upkeep pay-{4}-to-untap succeeds, so its
//   downside in the real game is repeated 1-damage draw-step pings while tapped. Adding
//   the mana ability without that downside grants free colorless mana with none of the card's
//   actual cost — wrong game state. Blocked until a draw-step trigger condition ships.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-vault"),
        name: "Mana Vault".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact doesn't untap during your untap step.\nAt the beginning of your upkeep, you may pay {4}. If you do, untap this artifact.\nAt the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.\n{T}: Add {C}{C}{C}.".to_string(),
        abilities: vec![
            // CR 502.3: "This artifact doesn't untap during your untap step."
            AbilityDefinition::Keyword(KeywordAbility::DoesNotUntap),
            // PB-AC2 (CR 118.12): "At the beginning of your upkeep, you may pay {4}.
            // If you do, untap this artifact."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourUpkeep,
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost { generic: 4, ..Default::default() }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::UntapPermanent { target: EffectTarget::Source }),
                },
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
