// Open the Vaults — {4}{W}{W} Sorcery
// Return all artifact and enchantment cards from all graveyards to the battlefield
// under their owners' control. (Auras with nothing to enchant remain in graveyards.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("open-the-vaults"),
        name: "Open the Vaults".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Return all artifact and enchantment cards from all graveyards to the battlefield under their owners' control. (Auras with nothing to enchant remain in graveyards.)".to_string(),
        abilities: vec![
            // TODO: "Return all artifact and enchantment cards from all graveyards to the
            // battlefield under their owners' control" requires:
            // 1. A ForEachTarget variant covering all graveyards (not just one player's).
            // 2. Filtering by multiple card types (Artifact OR Enchantment).
            // 3. Controller override to "owner" (not caster).
            // 4. Aura placement validation (Auras with nothing to enchant remain in graveyards).
            // None of these combinations are expressible in the current DSL. Empty per W5 policy.
        ],
        ..Default::default()
    }
}
