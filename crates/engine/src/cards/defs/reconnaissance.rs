// Reconnaissance — {W}, Enchantment
// {0}: Remove target attacking creature you control from combat and untap it.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("reconnaissance"),
        name: "Reconnaissance".to_string(),
        mana_cost: Some(ManaCost { white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "{0}: Remove target attacking creature you control from combat and untap it. (If you activate during end of combat, the creature will untap after it deals combat damage.)".to_string(),
        abilities: vec![
            // TODO: DSL gap — {0} activated ability that removes a target attacking creature
            // from combat (Effect::RemoveFromCombat) does not exist in the current DSL.
            // The untap portion is expressible (Effect::UntapPermanent) but remove-from-combat
            // is not.
        ],
        ..Default::default()
    }
}
