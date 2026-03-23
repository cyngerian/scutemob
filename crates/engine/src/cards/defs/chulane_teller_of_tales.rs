// Chulane, Teller of Tales — {2}{G}{W}{U}, Legendary Creature — Human Druid 2/4
// Vigilance
// Whenever you cast a creature spell, draw a card, then you may put a land card from
// your hand onto the battlefield.
// {3}, {T}: Return target creature you control to its owner's hand.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chulane-teller-of-tales"),
        name: "Chulane, Teller of Tales".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, white: 1, blue: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Druid"],
        ),
        oracle_text: "Vigilance\nWhenever you cast a creature spell, draw a card, then you may put a land card from your hand onto the battlefield.\n{3}, {T}: Return target creature you control to its owner's hand.".to_string(),
        power: Some(2),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // TODO: "Whenever you cast a creature spell" — WheneverYouCastSpell lacks
            // spell-type filter. Would need draw + optional land-from-hand-to-battlefield.
            // TODO: "{3}, {T}: Return target creature you control to its owner's hand."
            // Requires targeted activated ability with MoveZone to hand.
        ],
        ..Default::default()
    }
}
