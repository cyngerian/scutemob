// Breath of Fury -- {2}{R}{R}, Enchantment -- Aura
// Enchant creature you control.
// When enchanted creature deals combat damage to a player, sacrifice it and
// attach Breath of Fury to a creature you control. If you do, untap all
// creatures you control and after this phase, there is an additional combat phase.
//
// NOTE: This card requires Aura re-attachment (sacrifice enchanted creature,
// move Aura to another creature) and a conditional additional combat phase.
// Aura re-attachment is a DSL gap not yet implemented. The trigger body below
// is a placeholder with TODOs; the core AdditionalCombatPhase primitive is
// exercised by other cards (Karlach, Combat Celebrant).
//
// CR 702.5a: the printed Enchant line carries a controller clause. Declared as
//   EnchantFilter { has_card_type: Some(Creature), controller: You } (PB-DX20b).
//   Before PB-DX20b this def declared EnchantTarget::Creature, which dropped
//   "you control" — an opponent's creature was a legal target and a legal attachment.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    // CR 702.5a — "Enchant creature you control".
    let enchant_filter = EnchantFilter {
        has_card_type: Some(CardType::Creature),
        controller: EnchantControllerConstraint::You,
        ..Default::default()
    };
    CardDefinition {
        card_id: cid("breath-of-fury"),
        name: "Breath of Fury".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 2,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Enchantment], &["Aura"]),
        oracle_text: "Enchant creature you control\nWhen enchanted creature deals combat damage \
                      to a player, sacrifice it and attach Breath of Fury to a creature you \
                      control. If you do, untap all creatures you control and after this phase, \
                      there is an additional combat phase."
            .to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Enchant(EnchantTarget::Filtered(
                enchant_filter,
            ))),
            // TODO: Triggered ability -- when enchanted creature deals combat damage to a player:
            // 1. Sacrifice the enchanted creature.
            // 2. Attach this Aura to another creature you control (DSL gap: no re-attach effect).
            // 3. If successful: untap all creatures you control.
            // 4. If successful: add an additional combat phase.
            // DSL gaps: Aura re-attachment effect; sacrifice-enchanted-creature cost.
        ],
        completeness: Completeness::partial(
            "Triggered ability -- when enchanted creature deals combat damage to a player: 1. \
             Sacrifice the enchanted creature. 2....",
        ),
        ..Default::default()
    }
}
