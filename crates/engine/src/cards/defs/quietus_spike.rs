// Quietus Spike — {3}, Artifact — Equipment
// Equipped creature has deathtouch.
// Whenever equipped creature deals combat damage to a player, that player loses half their life, rounded up.
// Equip {3}
// TODO: DSL gap — "equipped creature has deathtouch" requires equipment continuous effect;
// the combat damage trigger with "half life rounded up" has no EffectAmount::HalfRoundedUp variant.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("quietus-spike"),
        name: "Quietus Spike".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has deathtouch.\nWhenever equipped creature deals combat damage to a player, that player loses half their life, rounded up.\nEquip {3}".to_string(),
        abilities: vec![],
        // TODO: equipped creature gains Deathtouch (continuous equipment effect)
        // TODO: combat damage trigger — target player loses half their life rounded up
        // TODO: Equip {3} activated ability
        ..Default::default()
    }
}
