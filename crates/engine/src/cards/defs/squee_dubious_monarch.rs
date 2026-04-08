// Squee, Dubious Monarch — {2}{R}, Legendary Creature — Goblin Noble 2/2
// Haste
// Whenever Squee attacks, create a 1/1 red Goblin creature token that's tapped and attacking.
// You may cast this card from your graveyard by paying {3}{R} and exiling four other cards
// from your graveyard rather than paying its mana cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("squee-dubious-monarch"),
        name: "Squee, Dubious Monarch".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Goblin", "Noble"]),
        oracle_text: "Haste\nWhenever Squee attacks, create a 1/1 red Goblin creature token that's tapped and attacking.\nYou may cast this card from your graveyard by paying {3}{R} and exiling four other cards from your graveyard rather than paying its mana cost.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // "Whenever Squee attacks, create a 1/1 red Goblin token that's tapped and attacking"
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        power: 1,
                        toughness: 1,
                        count: 1,
                        keywords: im::OrdSet::new(),
                        tapped: true,
                        enters_attacking: true,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 601.3: "You may cast this card from your graveyard by paying {3}{R} and
            // exiling four other cards from your graveyard rather than paying its mana cost."
            // Ruling 2022-09-09: If countered or dies after casting this way, it returns to
            // GY and may be cast again later.
            AbilityDefinition::CastSelfFromGraveyard {
                condition: None,
                alt_mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
                additional_costs: vec![CastFromGraveyardAdditionalCost::ExileOtherGraveyardCards(4)],
                required_alt_cost: None,
            },
        ],
        ..Default::default()
    }
}
