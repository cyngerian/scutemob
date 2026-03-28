// Kellogg, Dangerous Mind — {1}{B}{R}, Legendary Creature — Human Mercenary 3/2
// First strike, haste
// Whenever Kellogg attacks, create a Treasure token.
// Sacrifice five Treasures: Gain control of target creature for as long as you control
// Kellogg. Activate only as a sorcery.
//
// TODO: "Sacrifice five Treasures: Gain control of target creature for as long as you
// control Kellogg." — GainControl with duration "for as long as you control this" is not
// expressible in DSL (no EffectDuration::WhileYouControlSource or GainControl Effect).
// Cost::Sacrifice(N) for a specific count of a subtype also not in DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kellogg-dangerous-mind"),
        name: "Kellogg, Dangerous Mind".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 1, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Mercenary"]),
        oracle_text: "First strike, haste\nWhenever Kellogg attacks, create a Treasure token.\nSacrifice five Treasures: Gain control of target creature for as long as you control Kellogg. Activate only as a sorcery.".to_string(),
        power: Some(3),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::FirstStrike),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::CreateToken {
                    spec: treasure_token_spec(1),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // TODO: "Sacrifice five Treasures: Gain control of target creature for as long as
            // you control Kellogg." — see comment above.
        ],
        ..Default::default()
    }
}
