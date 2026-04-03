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
            // Whenever you cast a creature spell, draw a card, then you may put a land card
            // from your hand onto the battlefield.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WheneverYouCastSpell {
                    during_opponent_turn: false,
                    spell_type_filter: Some(vec![CardType::Creature]),
                    noncreature_only: false,
                },
                effect: Effect::Sequence(vec![
                    Effect::DrawCards {
                        player: PlayerTarget::Controller,
                        count: EffectAmount::Fixed(1),
                    },
                    Effect::PutLandFromHandOntoBattlefield { tapped: false },
                ]),
                intervening_if: None,
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
            // CR 602.2: "{3}, {T}: Return target creature you control to its owner's hand."
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 3, ..Default::default() }),
                    Cost::Tap,
                ]),
                effect: Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Hand { owner: PlayerTarget::OwnerOf(Box::new(EffectTarget::DeclaredTarget { index: 0 })) },
                    controller_override: None,
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
