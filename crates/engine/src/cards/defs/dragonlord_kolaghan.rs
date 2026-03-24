// Dragonlord Kolaghan — {4}{B}{R}, Legendary Creature — Elder Dragon 6/5
// Flying, haste
// Other creatures you control have haste.
// Whenever an opponent casts a creature or planeswalker spell with the same name as a card in their graveyard, that player loses 10 life.
// TODO: DSL gap — the triggered ability requires checking the opponent's graveyard for a name match,
// which is not supported by any TriggerCondition in the current DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("dragonlord-kolaghan"),
        name: "Dragonlord Kolaghan".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 1, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Elder", "Dragon"],
        ),
        oracle_text: "Flying, haste\nOther creatures you control have haste.\nWhenever an opponent casts a creature or planeswalker spell with the same name as a card in their graveyard, that player loses 10 life.".to_string(),
        power: Some(6),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // CR 604.2 / CR 613.1f: "Other creatures you control have haste."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: triggered — opponent casts a spell with same name as a card in their graveyard → loses 10 life.
            // DSL gap: no TriggerCondition checking opponent's graveyard for name match.
        ],
        ..Default::default()
    }
}
