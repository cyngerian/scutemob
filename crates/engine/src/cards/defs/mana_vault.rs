// Mana Vault — {1} Artifact
// This artifact doesn't untap during your untap step.
// At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact.
// At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.
// {T}: Add {C}{C}{C}.
//
// ENGINE-BLOCKED: "At the beginning of your upkeep, you may pay {4}. If you do, untap this
//   artifact." — optional-cost triggered ability not in DSL (blocked on PB-AC2).
// ENGINE-BLOCKED: "At the beginning of your draw step, if this artifact is tapped, it deals
//   1 damage to you." — draw-step trigger with a tapped intervening-if condition not in DSL.
// ENGINE-BLOCKED: "{T}: Add {C}{C}{C}." — per W5 policy (KI-13 class), this mana ability's
//   real cost/risk is inseparable from the draw-step damage-if-tapped clause above: once
//   tapped, Mana Vault stays tapped forever (`DoesNotUntap`, and the pay-{4}-to-untap path is
//   also blocked), so its downside in the real game is repeated 1-damage upkeep pings. Adding
//   the mana ability without that downside grants free colorless mana with none of the card's
//   actual cost — wrong game state. Blocked until the draw-step damage trigger above ships.
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
        ],
        ..Default::default()
    }
}
