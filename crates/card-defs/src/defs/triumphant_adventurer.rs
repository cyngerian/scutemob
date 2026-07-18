// Triumphant Adventurer — {W}{B}, Creature — Human Knight 1/1
// Deathtouch
// During your turn, this creature has first strike.
// Whenever this creature attacks, venture into the dungeon.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("triumphant-adventurer"),
        name: "Triumphant Adventurer".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            black: 1,
            ..Default::default()
        }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Deathtouch\nDuring your turn, this creature has first strike.\nWhenever \
                      this creature attacks, venture into the dungeon. (Enter the first room or \
                      advance to the next room.)"
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // CR 604.2 / 613.1f (Layer 6): "During your turn, this creature has first strike."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::FirstStrike),
                    filter: EffectFilter::Source,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::IsYourTurn),
                },
            },
            // CR 701.49a-c: attack trigger — venture into the dungeon.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::VentureIntoDungeon,
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
