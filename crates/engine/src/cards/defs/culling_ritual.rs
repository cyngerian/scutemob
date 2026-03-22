// Culling Ritual — {2}{B}{G} Sorcery
// Destroy each nonland permanent with mana value 2 or less.
// Add {B} or {G} for each permanent destroyed this way.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("culling-ritual"),
        name: "Culling Ritual".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text:
            "Destroy each nonland permanent with mana value 2 or less. Add {B} or {G} for each permanent destroyed this way."
                .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 701.8: Destroy all nonland permanents with MV 2 or less.
            // The mana addition (one {B} or {G} per permanent) requires per-destroyed-count mana
            // scaling with a player choice of color per mana — EffectAmount::LastEffectCount
            // with Choose/AddMana cannot encode the per-instance B-or-G selection.
            // TODO: When variable-count mana generation with color choice is supported,
            // add Effect::Choose wrapping AddMana(B) / AddMana(G) scaled by LastEffectCount.
            effect: Effect::DestroyAll {
                filter: TargetFilter {
                    non_land: true,
                    max_cmc: Some(2),
                    ..Default::default()
                },
                cant_be_regenerated: false,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
