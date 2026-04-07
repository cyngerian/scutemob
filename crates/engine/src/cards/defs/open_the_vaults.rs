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
            // CR 400.7, 603.6a: Return all artifact and enchantment cards from all graveyards
            // to the battlefield under their owners' control simultaneously.
            // NOTE: "Auras with nothing to enchant remain in graveyards" (CR 704.5m):
            // Auras enter unattached and are put into the graveyard by the next SBA check.
            // Interactive Aura placement (choosing what to enchant as they enter) is deferred
            // to M10+ (requires player choice infrastructure). This is a known approximation.
            // TODO(M10+): Add Aura placement choice so Auras can attach to valid targets.
            AbilityDefinition::Spell {
                effect: Effect::ReturnAllFromGraveyardToBattlefield {
                    graveyards: PlayerTarget::EachPlayer,
                    filter: TargetFilter {
                        has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                        ..Default::default()
                    },
                    tapped: false,
                    controller_override: None, // "under their owners' control"
                    unique_names: false,
                    permanent_cards_only: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
