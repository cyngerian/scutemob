// Prosperous Innkeeper — Alliance (life gain), ETB Treasure token
// CR 702.x: Alliance is an ability word (no keyword variant); implemented as a
// plain Triggered ability using WheneverCreatureEntersBattlefield with
// controller: You filter. The engine's trigger collector sets exclude_self: true
// automatically for all WheneverCreatureEntersBattlefield triggers.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("prosperous-innkeeper"),
        name: "Prosperous Innkeeper".to_string(),
        mana_cost: Some(ManaCost { generic: 1, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Halfling", "Citizen"]),
        oracle_text: "When this creature enters, create a Treasure token. (It's an artifact with \"{T}, Sacrifice this token: Add one mana of any color.\")\nAlliance — Whenever another creature you control enters, you gain 1 life.".to_string(),
        abilities: vec![
            // CR 603.1: ETB trigger — create a Treasure token.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::CreateToken { spec: treasure_token_spec(1) },
                intervening_if: None,
                targets: vec![],
            },
            // Alliance ability word (CR 702 ability word — no KeywordAbility variant).
            // Fires whenever another creature you control enters (exclude_self applied by engine).
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverCreatureEntersBattlefield {
                    filter: Some(TargetFilter {
                        controller: TargetController::You,
                        ..Default::default()
                    }),
                },
                effect: Effect::GainLife {
                    player: PlayerTarget::Controller,
                    amount: EffectAmount::Fixed(1),
                },
                intervening_if: None,
                targets: vec![],
            },
        ],
        power: Some(1),
        toughness: Some(1),
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
    }
}
