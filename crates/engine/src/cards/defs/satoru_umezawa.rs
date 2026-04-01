// Satoru Umezawa — {1}{U}{B}, Legendary Creature — Human Ninja 2/4
// Whenever you activate a ninjutsu ability, look at the top three cards of your library.
//   Put one of them into your hand and the rest on the bottom of your library in any order.
//   This ability triggers only once each turn.
// Each creature card in your hand has ninjutsu {2}{U}{B}.
//
// TODO: TriggerCondition::WheneverYouActivateNinjutsu does not exist in DSL.
//   The ninjutsu activation trigger cannot be expressed. Omitted.
// TODO: "Look at top three cards, put one into hand, rest on bottom" — no LookAtTopN with
//   SelectAndKeep effect variant. Would need Effect::RevealAndRoute or similar.
// TODO: "Each creature card in your hand has ninjutsu {2}{U}{B}" — static ability granting
//   ninjutsu to cards in hand (not on battlefield) not supported by DSL.
//   EffectFilter::CreaturesYouControl only covers battlefield, not hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("satoru-umezawa"),
        name: "Satoru Umezawa".to_string(),
        mana_cost: Some(ManaCost { generic: 1, blue: 1, black: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Ninja"]),
        oracle_text: "Whenever you activate a ninjutsu ability, look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order. This ability triggers only once each turn.\nEach creature card in your hand has ninjutsu {2}{U}{B}.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            // TODO: WheneverYouActivateNinjutsu trigger — DSL gap (no such TriggerCondition).
            // TODO: look-at-top-3-put-1-in-hand — DSL gap (no SelectFromTopN effect).
            // TODO: static grant ninjutsu to cards in hand — DSL gap (EffectFilter::InHand not supported).
        ],
        ..Default::default()
    }
}
