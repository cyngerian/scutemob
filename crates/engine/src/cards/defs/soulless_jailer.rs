// Soulless Jailer — {2}, Artifact Creature — Phyrexian Golem 0/4
// Permanent cards in graveyards can't enter the battlefield.
// Players can't cast noncreature spells from graveyards or exile.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("soulless-jailer"),
        name: "Soulless Jailer".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(&[], &[CardType::Artifact, CardType::Creature], &["Phyrexian", "Golem"]),
        oracle_text: "Permanent cards in graveyards can't enter the battlefield.\nPlayers can't cast noncreature spells from graveyards or exile.".to_string(),
        power: Some(0),
        toughness: Some(4),
        abilities: vec![
            // TODO: Two static restrictions not in DSL:
            // 1. "Permanent cards in graveyards can't enter the battlefield" —
            //    needs GameRestriction::PermanentsCantEnterFromGraveyard.
            // 2. "Players can't cast noncreature spells from graveyards or exile" —
            //    needs GameRestriction::CantCastNoncreatureFromGraveyardOrExile.
        ],
        ..Default::default()
    }
}
