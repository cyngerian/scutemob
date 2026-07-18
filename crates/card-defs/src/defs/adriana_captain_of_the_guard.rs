// Adriana, Captain of the Guard — {3}{R}{W}, Legendary Creature — Human Knight 4/4
// Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each
// opponent you attacked this combat.)
// Other creatures you control have melee. (If a creature has multiple instances of
// melee, each triggers separately.)
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("adriana-captain-of-the-guard"),
        name: "Adriana, Captain of the Guard".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Knight"],
        ),
        oracle_text: "Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for \
                      each opponent you attacked this combat.)\nOther creatures you control have \
                      melee. (If a creature has multiple instances of melee, each triggers \
                      separately.)"
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            // CR 702.121a: printed Melee (self).
            AbilityDefinition::Keyword(KeywordAbility::Melee),
            // CR 613.1f / Layer 6: "Other creatures you control have melee."
            // The derived attack trigger for each granted Melee is synthesized at
            // layer-resolution time (PB-EF3b, layers::calculate_characteristics).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Melee),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
