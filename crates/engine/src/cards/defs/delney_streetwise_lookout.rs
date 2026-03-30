// Delney, Streetwise Lookout — {2}{W}, Legendary Creature — Human Scout 2/2
// Creatures you control with power 2 or less can't be blocked by creatures with
// power 3 or greater.
// If an ability of a creature you control with power 2 or less triggers, that
// ability triggers an additional time.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("delney-streetwise-lookout"),
        name: "Delney, Streetwise Lookout".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Scout"],
        ),
        oracle_text: "Creatures you control with power 2 or less can't be blocked by creatures with power 3 or greater.\nIf an ability of a creature you control with power 2 or less triggers, that ability triggers an additional time.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: Power-conditional blocking restriction — "can't be blocked by
            // creatures with power 3 or greater" applies only to your creatures with
            // power <= 2. Needs CantBeBlockedExceptBy with power filter on both
            // attacker and blocker.
            // TODO: Trigger doubling for low-power creatures — needs a power-filtered
            // TriggerDoubler variant.
        ],
        ..Default::default()
    }
}
