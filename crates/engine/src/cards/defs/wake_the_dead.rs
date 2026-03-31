// Wake the Dead — {X}{B}{B} Instant
// Cast this spell only during combat on an opponent's turn.
// Return X target creature cards from your graveyard to the battlefield.
// Sacrifice those creatures at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wake-the-dead"),
        name: "Wake the Dead".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        // {X}{B}{B} — X cost not expressible in ManaCost struct
        types: types(&[CardType::Instant]),
        oracle_text: "Cast this spell only during combat on an opponent's turn.\nReturn X target creature cards from your graveyard to the battlefield. Sacrifice those creatures at the beginning of the next end step.".to_string(),
        abilities: vec![
            // TODO: This card has multiple DSL gaps:
            // 1. {X} in mana cost: ManaCost has no X field; X-value effects require EffectAmount::XValue
            //    but the mana cost itself can't represent {X}{B}{B} properly.
            // 2. "Cast only during combat on an opponent's turn" — a timing restriction that is
            //    neither SorcerySpeed nor InstantSpeed. No TimingRestriction variant exists for this.
            // 3. "Return X target creature cards" — variable number of targets based on X.
            // 4. "Sacrifice those creatures at the beginning of the next end step" — delayed
            //    triggered sacrifice is not expressible in the DSL.
            // Empty per W5 policy.
        ],
        ..Default::default()
    }
}
