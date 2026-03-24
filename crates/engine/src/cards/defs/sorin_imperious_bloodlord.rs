// Sorin, Imperious Bloodlord — {2}{B}, Legendary Planeswalker — Sorin (loyalty 4)
// +1: Target creature you control gains deathtouch and lifelink until end of turn.
//     If it's a Vampire, put a +1/+1 counter on it.
// +1: You may sacrifice a Vampire. When you do, Sorin deals 3 damage to any target
//     and you gain 3 life.
// −3: You may put a Vampire creature card from your hand onto the battlefield.
//
// NOTE: First +1 "If it's a Vampire" conditional counter on the target requires checking the
// subtype of the declared target at resolution. Condition::TargetHasSubtype not in DSL.
// Implement the deathtouch/lifelink grants; omit the conditional counter per W5 policy.
// NOTE: Second +1 "You may sacrifice a Vampire. When you do, ..." requires a may-sacrifice
// cost with a "when you do" delayed trigger. Cost::Sacrifice(Goblin filter) exists but the
// conditional "when you do" response chain is not expressible. Implement as Sequence of
// sacrifice + damage + life gain (always fires if sacrifice occurs) per closest approximation.
// Actually — per W5, "you may sacrifice a Vampire; WHEN YOU DO" means the effects only happen
// IF a sacrifice occurs. This is not conditionally expressible without a CanDo/Optional cost.
// Use TODO for the second +1 ability per W5 policy.
// NOTE: −3 "put a Vampire creature card from your hand onto the battlefield" requires
// selecting a card from hand by subtype — no DSL expression for choosing from hand.
// Omitted per W5 policy.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sorin-imperious-bloodlord"),
        name: "Sorin, Imperious Bloodlord".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Sorin"],
        ),
        oracle_text: "+1: Target creature you control gains deathtouch and lifelink until end of turn. If it's a Vampire, put a +1/+1 counter on it.\n+1: You may sacrifice a Vampire. When you do, Sorin deals 3 damage to any target and you gain 3 life.\n\u{2212}3: You may put a Vampire creature card from your hand onto the battlefield.".to_string(),
        starting_loyalty: Some(4),
        abilities: vec![
            // +1: Target creature you control gains deathtouch and lifelink until end of turn.
            // (Conditional counter on Vampires omitted — Condition::TargetHasSubtype not in DSL.)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Deathtouch),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Lifelink),
                            filter: EffectFilter::DeclaredTarget { index: 0 },
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    // TODO: "If it's a Vampire, put a +1/+1 counter on it" — requires
                    // Condition::TargetHasSubtype(SubType("Vampire")) which is not in the DSL.
                    // Omitted per W5 policy.
                ]),
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
            },
            // +1: You may sacrifice a Vampire. When you do, Sorin deals 3 damage to any target
            // and you gain 3 life.
            // TODO: "You may sacrifice a Vampire. When you do, [effects]" — optional sacrifice
            // with conditional response chain is not expressible in the DSL. The Cost::Sacrifice
            // filter exists but there is no "may" (optional) variant and no "when you do" guard
            // on the subsequent effects. Omitted per W5 policy.

            // −3: You may put a Vampire creature card from your hand onto the battlefield.
            // TODO: Interactive hand selection by creature subtype ("Vampire creature card from
            // your hand") is not expressible in the DSL. Omitted per W5 policy.
        ],
        ..Default::default()
    }
}
