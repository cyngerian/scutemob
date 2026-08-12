// Sting, the Glinting Dagger — {2}, Legendary Artifact — Equipment
// RE-VERIFIED 2026-08-11 (PB-DX26 fix cycle, review Finding 4). Three of the four clauses
// are expressible TODAY and the old TODOs here claimed otherwise:
//   * "+1/+1 and has haste" — `EffectFilter::AttachedCreature` is the equipment filter and
//     it exists (the old note guessed at an `EffectFilter::EquippedCreature` that never did).
//   * "At the beginning of each combat, untap equipped creature" — expressible.
//   * "Equip {2}" — expressible; PB-DX26 authored this exact shape into 21 other defs.
// TODO: still genuinely blocked — "Equipped creature has first strike as long as it's
//   blocking or blocked by a Goblin or Orc": no `Condition` expresses a combat relationship
//   to a creature of a given subtype (re-checked against the current enum 2026-08-11).
// The whole card stays WITHHELD (`Completeness::inert`, no abilities) under W5/W6 rather
// than shipping three clauses and dropping a combat-relevant keyword. See `OOS-DX26-1`.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sting-the-glinting-dagger"),
        name: "Sting, the Glinting Dagger".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "Equipped creature gets +1/+1 and has haste.\nAt the beginning of each \
                      combat, untap equipped creature.\nEquipped creature has first strike as \
                      long as it's blocking or blocked by a Goblin or Orc.\nEquip {2}"
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Three of four clauses are now expressible (AttachedCreature static for +1/+1+haste, \
             AtBeginningOfCombat+EquippedCreature untap, Equip {2}). Blocked only on 'first \
             strike as long as it's blocking or blocked by a Goblin or Orc' — no Condition \
             variant for a combat relationship to a creature of a given subtype. Authoring the \
             rest without it would drop a combat-relevant keyword (wrong game state per W6).",
        ),
        ..Default::default()
    }
}
