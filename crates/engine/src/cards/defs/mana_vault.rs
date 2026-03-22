// Mana Vault — {1} Artifact; doesn't untap; upkeep pay {4} to untap; draw step damage if tapped;
// {T}: Add {C}{C}{C}.
// TODO: "This artifact doesn't untap during your untap step" — no CantUntap static effect.
// TODO: "At the beginning of your upkeep, you may pay {4}. If you do, untap this artifact." —
//   conditional untap triggered ability not in DSL.
// TODO: "At the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you." —
//   draw-step trigger with tapped intervening-if condition not in DSL.
// W5 policy: implementing only the tap ability would let Mana Vault produce {C}{C}{C} without
// any cost or restriction — wrong game state without the doesn't-untap constraint.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mana-vault"),
        name: "Mana Vault".to_string(),
        mana_cost: Some(ManaCost { generic: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "This artifact doesn't untap during your untap step.\nAt the beginning of your upkeep, you may pay {4}. If you do, untap this artifact.\nAt the beginning of your draw step, if this artifact is tapped, it deals 1 damage to you.\n{T}: Add {C}{C}{C}.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
