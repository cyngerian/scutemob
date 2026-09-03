// Kaya's Ghostform — {B}, Enchantment — Aura
// Enchant creature or planeswalker you control
// When enchanted permanent dies or is put into exile, return that card to the battlefield
// under your control.
//
// CR 702.5a: the printed Enchant line is an OR over two CARD TYPES plus a controller
//   clause. Declared as EnchantFilter { has_card_types: [Creature, Planeswalker],
//   controller: You } (PB-DX20b). Before PB-DX20b `EnchantFilter` had no OR over card
//   types, so this def declared EnchantTarget::Creature, which wrongly narrowed the legal
//   set to creatures and dropped "you control" entirely (OOS-DX20-5).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    // CR 702.5a — "Enchant creature or planeswalker you control".
    let enchant_filter = EnchantFilter {
        has_card_types: vec![CardType::Creature, CardType::Planeswalker],
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("kayas-ghostform"),
        name: "Kaya's Ghostform".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature or planeswalker you control\nWhen enchanted permanent dies \
                      or is put into exile, return that card to the battlefield under your \
                      control."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // TODO: DSL gap — "When enchanted permanent dies or is exiled, return that card
            // to the battlefield" needs a trigger keyed to the ENCHANTED permanent's zone
            // change (WhenDies / WhenLeavesBattlefield key on the Aura itself), plus a
            // return from graveyard-or-exile. The Enchant line itself is NOT a gap: it is
            // expressed above (PB-DX20b).
        ],
        completeness: Completeness::partial(
            "Blocked on a trigger keyed to the ENCHANTED permanent's zone change ('dies or is put \
             into exile') and on returning that card from graveyard-or-exile — \
             WhenDies/WhenLeavesBattlefield key on the Aura itself. The Enchant line is NOT a \
             blocker: PB-DX20b added EnchantFilter::has_card_types, so 'Enchant creature or \
             planeswalker you control' is declared exactly as printed.",
        ),
        ..Default::default()
    }
}
