// Mass Hysteria — {R}, Enchantment
// All creatures have haste.
// TODO: DSL gap — "all creatures have haste" is a static continuous effect (layer 6) granting
// haste to all creatures on the battlefield including opponents'; no EffectFilter::AllCreatures
// granting keyword as a static card ability exists (only EffectFilter::AllCreatures is used
// in Effect::DestroyPermanent context, not in ContinuousEffectDef).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("mass-hysteria"),
        name: "Mass Hysteria".to_string(),
        mana_cost: Some(ManaCost { red: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "All creatures have haste.".to_string(),
        abilities: vec![
            // TODO: static — all creatures have haste (layer 6 continuous effect).
            // DSL gap: no static ContinuousEffectDef with EffectFilter::AllCreatures granting keywords.
        ],
        ..Default::default()
    }
}
