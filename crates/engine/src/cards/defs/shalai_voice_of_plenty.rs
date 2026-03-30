// Shalai, Voice of Plenty — {3}{W}, Legendary Creature — Angel 3/4
// Flying
// You, planeswalkers you control, and other creatures you control have hexproof.
// {4}{G}{G}: Put a +1/+1 counter on each creature you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shalai-voice-of-plenty"),
        name: "Shalai, Voice of Plenty".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Angel"],
        ),
        oracle_text: "Flying\nYou, planeswalkers you control, and other creatures you control have hexproof.\n{4}{G}{G}: Put a +1/+1 counter on each creature you control.".to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // Layer 6: other creatures you control have hexproof.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Hexproof),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: DSL gap — "You have hexproof" (player hexproof) not in layer system.
            // TODO: DSL gap — "planeswalkers you control have hexproof" (planeswalker filter).
            // {4}{G}{G}: Put a +1/+1 counter on each creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 4, green: 2, ..Default::default() }),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreatureYouControl,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        ..Default::default()
    }
}
