// Transcendent Dragon — {4}{U}{U}, Creature — Dragon 4/3
// Flash
// Flying
// When this creature enters, if you cast it, counter target spell. If that spell is
// countered this way, exile it instead of putting it into its owner's graveyard, then
// you may cast it without paying its mana cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("transcendent-dragon"),
        name: "Transcendent Dragon".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 2, ..Default::default() }),
        types: creature_types(&["Dragon"]),
        oracle_text: "Flash\nFlying\nWhen this creature enters, if you cast it, counter target spell. If that spell is countered this way, exile it instead of putting it into its owner's graveyard, then you may cast it without paying its mana cost.".to_string(),
        power: Some(4),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flash),
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: "When this creature enters, if you cast it, counter target spell."
            // + exile-instead + free cast from exile. Requires "if you cast it" intervening-if
            // (Condition::WasCast) + counter-to-exile + PlayExiledCard. Without intervening-if,
            // this would fire on reanimation/flicker (KI-2). Stripped per W6 policy.
        ],
        ..Default::default()
    }
}
