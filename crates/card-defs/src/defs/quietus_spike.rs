// Quietus Spike — {3}, Artifact — Equipment
// Equipped creature has deathtouch.
// Whenever equipped creature deals combat damage to a player, that player loses half their life, rounded up.
// Equip {3}
// RE-VERIFIED 2026-08-11 (PB-DX26 fix cycle, review Finding 4): the deathtouch grant IS
// expressible (`AddKeywords` + `EffectFilter::AttachedCreature`, cf. `basilisk_collar.rs`),
// and so is `Equip {3}` — PB-DX26 authored that exact shape into 21 other defs.
// TODO: still genuinely blocked — "that player loses half their life, rounded up" has no
//   `EffectAmount` half-rounded-up variant (re-checked 2026-08-11).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("quietus-spike"),
        name: "Quietus Spike".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has deathtouch.\nWhenever equipped creature deals combat \
                      damage to a player, that player loses half their life, rounded up.\nEquip \
                      {3}"
        .to_string(),
        abilities: vec![],
        // The whole card is WITHHELD under W5/W6 rather than shipping deathtouch + equip
        // with the life loss silently dropped. Deathtouch, the trigger condition and
        // Equip {3} are all expressible today; only the half-rounded-up amount is not.
        // See the header note and `OOS-DX26-1`.
        completeness: Completeness::inert(
            "Deathtouch grant (AddKeywords + EffectFilter::AttachedCreature, cf. \
             basilisk_collar.rs) and Equip {3} are expressible today, as is the \
             WhenEquippedCreatureDealsCombatDamageToPlayer trigger. Blocked solely on the \
             trigger's effect: EffectAmount has no half-rounded-up variant for 'that player loses \
             half their life, rounded up'. Withheld per W5 rather than shipping deathtouch+equip \
             with the life-loss silently dropped.",
        ),
        ..Default::default()
    }
}
