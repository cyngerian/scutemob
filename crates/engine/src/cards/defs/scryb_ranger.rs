// Scryb Ranger — {1}{G}, Creature — Faerie Ranger 1/1
// Flash
// Flying, protection from blue
// Return a Forest you control to its owner's hand: Untap target creature. Activate only once each turn.
//
// Flash, Flying, and Protection from blue are implemented.
// TODO: DSL gap — the activated ability (return Forest to hand: untap target creature, once per turn)
// requires a Cost::ReturnPermanentToHand filter (Forest type), Effect::UntapPermanent, and
// a once-per-turn activation restriction. None of these exist in the DSL.
use crate::cards::helpers::*;
use crate::state::types::ProtectionQuality;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scryb-ranger"),
        name: "Scryb Ranger".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Faerie", "Ranger"]),
        oracle_text: "Flash\nFlying, protection from blue\nReturn a Forest you control to its owner's hand: Untap target creature. Activate only once each turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::Blue),
            )),
            // TODO: Return a Forest to hand: Untap target creature (once per turn)
            // — Cost::ReturnPermanentToHand not in DSL; Effect::UntapPermanent not in DSL
        ],
        ..Default::default()
    }
}
