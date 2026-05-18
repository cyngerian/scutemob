// Crossway Troublemakers — {5}{B}, Creature — Vampire 5/5
// Attacking Vampires you control have deathtouch and lifelink.
// Whenever a Vampire you control dies, you may pay 2 life. If you do, draw a card.
//
// ENGINE-BLOCKED (draw clause): "you may pay 2 life — if you do, draw a card" requires
// a beneficial optional-cost wrapper (Effect::MayPayThenEffect). Effect::MayPayOrElse
// expresses "pay or suffer", NOT "pay to gain benefit". Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("crossway-troublemakers"),
        name: "Crossway Troublemakers".to_string(),
        mana_cost: Some(ManaCost { generic: 5, black: 1, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Attacking Vampires you control have deathtouch and lifelink.\nWhenever a Vampire you control dies, you may pay 2 life. If you do, draw a card.".to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // CR 613.1f / CR 611.3a: "Attacking Vampires you control have deathtouch."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Vampire".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // CR 613.1f / CR 611.3a: "Attacking Vampires you control have lifelink."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                    filter: EffectFilter::AttackingCreaturesYouControlWithSubtype(
                        SubType("Vampire".to_string()),
                    ),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // ENGINE-BLOCKED: "Whenever a Vampire you control dies, you may pay 2 life.
            // If you do, draw a card." — the trigger fires correctly (Vampire filter now
            // supported), but the optional-pay-to-draw clause has no DSL primitive.
            // Effect::MayPayOrElse expresses a tax (pay or suffer), not a benefit rider.
            // Omitting the entire triggered ability rather than approximating with
            // unconditional draw (which would produce wrong game state).
        ],
        ..Default::default()
    }
}
