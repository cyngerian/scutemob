// Chrome Mox — {0} Artifact; Imprint ETB exile from hand; tap for exiled card's color.
// TODO: ETB optional exile from hand (Imprint) and "add mana of exiled card's colors"
// are both DSL gaps (no EffectTarget::ImprintedCard or ETB choice-exile-from-hand primitive).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chrome-mox"),
        name: "Chrome Mox".to_string(),
        mana_cost: Some(ManaCost { ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Imprint — When this artifact enters, you may exile a nonartifact, nonland card from your hand.\n{T}: Add one mana of any of the exiled card's colors.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
