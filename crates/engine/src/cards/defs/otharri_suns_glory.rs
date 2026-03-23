// Otharri, Suns' Glory — {3}{R}{W}, Legendary Creature — Phoenix 3/3
// Flying, lifelink, haste
// Whenever Otharri attacks, you get an experience counter. Then create a 2/2 red Rebel
// creature token that's tapped and attacking for each experience counter you have.
// {2}{R}{W}, Tap an untapped Rebel you control: Return this card from your graveyard
// to the battlefield tapped.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("otharri-suns-glory"),
        name: "Otharri, Suns' Glory".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, white: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Phoenix"]),
        oracle_text: "Flying, lifelink, haste\nWhenever Otharri attacks, you get an experience counter. Then create a 2/2 red Rebel creature token that's tapped and attacking for each experience counter you have.\n{2}{R}{W}, Tap an untapped Rebel you control: Return this card from your graveyard to the battlefield tapped.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: "Whenever Otharri attacks, you get an experience counter. Then create a 2/2
            // red Rebel token that's tapped and attacking for each experience counter you have."
            // — experience counters on players not in DSL (PlayerState has no experience_counters
            // field). Also, EffectAmount lacks a player-counter-count variant for the loop.
            // Per W5 policy, leaving attack trigger empty to avoid wrong game state.

            // TODO: "{2}{R}{W}, Tap an untapped Rebel you control: Return this card from your
            // graveyard to the battlefield tapped." — activated ability from graveyard with
            // Tap-a-Rebel cost not expressible (Cost::TapPermanentWithSubtype not in DSL).
            // Also no ReturnFromGraveyardToBattlefield effect with tapped modifier.
        ],
        ..Default::default()
    }
}
