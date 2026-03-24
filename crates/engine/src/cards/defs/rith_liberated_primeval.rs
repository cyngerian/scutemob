// Rith, Liberated Primeval — {2}{R}{G}{W}, Legendary Creature — Dragon 5/5
// Flying, ward {2}
// Other Dragons you control have ward {2}.
// At the beginning of your end step, if a creature or planeswalker an opponent controlled
// was dealt excess damage this turn, create a 4/4 red Dragon creature token with flying.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("rith-liberated-primeval"),
        name: "Rith, Liberated Primeval".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, green: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying, ward {2}\nOther Dragons you control have ward {2}.\nAt the beginning of your end step, if a creature or planeswalker an opponent controlled was dealt excess damage this turn, create a 4/4 red Dragon creature token with flying.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Ward(2)),
            // CR 613.1f (Layer 6): "Other Dragons you control have ward {2}."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Ward(2)),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(
                        SubType("Dragon".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "At the beginning of your end step, if a creature or planeswalker an
            // opponent controlled was dealt excess damage this turn, create a 4/4 Dragon token."
            // Blocked: "excess damage this turn" intervening-if condition not in DSL.
        ],
        ..Default::default()
    }
}
