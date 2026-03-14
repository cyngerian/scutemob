// Flare of Fortitude — {2}{W}{W}, Instant
// You may sacrifice a nontoken white creature rather than pay this spell's mana cost.
// Until end of turn, your life total can't change, and permanents you control gain hexproof and indestructible.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("flare-of-fortitude"),
        name: "Flare of Fortitude".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "You may sacrifice a nontoken white creature rather than pay this spell's mana cost.\nUntil end of turn, your life total can't change, and permanents you control gain hexproof and indestructible.".to_string(),
        abilities: vec![
            // TODO: Alt cost — sacrifice a nontoken white creature (not PB-4 activated cost)
            // Needs AltCostKind::SacrificeCreature variant with token/color filter.
            // TODO: Spell effect — until end of turn, your life total can't change, and permanents
            // you control gain hexproof and indestructible.
            // DSL gap: no "life total can't change" effect; no mass hexproof+indestructible grant.
        ],
        ..Default::default()
    }
}
