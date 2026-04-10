// Umbral Mantle — {3}, Artifact — Equipment
// Equipped creature has "{3}, {Q}: This creature gets +2/+2 until end of turn."
// Equip {0}
//
// Partially unblocked by PB-S: the grant uses AddActivatedAbility with
//   EffectFilter::AttachedCreature. Still blocked on:
//   (1) {Q} (untap symbol) — ActivationCost lacks requires_untap_self field
//   (2) self-pump effect ("this creature gets +2/+2 until EOT") is expressible
//       but needs the {Q} cost to be complete
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("umbral-mantle"),
        name: "Umbral Mantle".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature has \"{3}, {Q}: This creature gets +2/+2 until end of turn.\"\nEquip {0}".to_string(),
        abilities: vec![
            // TODO: grant "3, {Q}: gets +2/+2 until EOT" to equipped creature via
            //   LayerModification::AddActivatedAbility + EffectFilter::AttachedCreature.
            //   Blocked on {Q} (untap symbol) — ActivationCost needs requires_untap_self.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
