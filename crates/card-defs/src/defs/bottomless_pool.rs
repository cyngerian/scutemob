// Bottomless Pool // Locker Room — When you unlock this door, return up to one target creature to its own
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bottomless-pool"),
        name: "Bottomless Pool // Locker Room".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: types_sub(&[CardType::Enchantment], &["Room"]),
        oracle_text: "When you unlock this door, return up to one target creature to its owner's hand.\n(You may cast either half. That door unlocks on the battlefield. As a sorcery, you may pay the mana cost of a locked door to unlock it.)".to_string(),
        abilities: vec![],
        completeness: Completeness::inert("Blocked on the Room / door-unlock mechanic (CR 725): no 'when you unlock this door' trigger event, no unlock action, and no representation for a two-door split enchantment with per-door mana costs. The effect body ('return up to one target creature to its owner's hand') is expressible today via TargetRequirement::UpToN + MoveZone to Hand with PlayerTarget::OwnerOf. Shares this blocker with funeral_room.rs."),
        ..Default::default()
    }
}
