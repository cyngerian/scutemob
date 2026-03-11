// Revitalizing Repast // Old-Growth Grove — Put a +1/+1 counter on target creature. It gains indestructible until 
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("revitalizing-repast"),
        name: "Revitalizing Repast // Old-Growth Grove".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put a +1/+1 counter on target creature. It gains indestructible until end of turn.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
