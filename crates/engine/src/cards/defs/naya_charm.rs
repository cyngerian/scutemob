// Naya Charm — {R}{G}{W} Instant; Choose one —
// • Deal 3 damage to target creature.
// • Return target card from a graveyard to its owner's hand.
// • Tap all creatures target player controls.
//
// TODO: DSL gap — Mode 3 ("Tap all creatures target player controls") requires
// TapPermanent targeting creatures filtered by a declared target player's control.
// EffectTarget has no AllCreaturesControlledBy(PlayerTarget) variant; AllCreatures
// would tap ALL creatures and produce wrong game state. Full modal ability deferred
// until that target variant is available.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("naya-charm"),
        name: "Naya Charm".to_string(),
        mana_cost: Some(ManaCost { red: 1, green: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Choose one —\n• Naya Charm deals 3 damage to target creature.\n• Return target card from a graveyard to its owner's hand.\n• Tap all creatures target player controls.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
