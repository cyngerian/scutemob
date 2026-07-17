// Gilded Drake — {1}{U}, Creature — Drake 3/3
// Flying
// When this creature enters, exchange control of this creature and up to one target
// creature an opponent controls. If you don't or can't make an exchange, sacrifice
// this creature. This ability still resolves if its target becomes illegal.
//
// PARTIAL: Flying + ETB ExchangeControl with UpToN(1) opponent-creature target authored.
// ENGINE-BLOCKED: "If you don't or can't make an exchange, sacrifice this creature." —
// there is no Condition variant expressing whether the exchange occurred (no target was
// declared, or the target became illegal at resolution). Conditional self-sacrifice omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gilded-drake"),
        name: "Gilded Drake".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Drake"]),
        oracle_text: "Flying\nWhen this creature enters, exchange control of this creature and up \
                      to one target creature an opponent controls. If you don't or can't make an \
                      exchange, sacrifice this creature. This ability still resolves if its \
                      target becomes illegal."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // ETB: exchange control of this creature and up to one target creature an opponent controls.
            // ENGINE-BLOCKED: conditional self-sacrifice ("if you don't or can't make an exchange")
            // requires a Condition for whether the exchange actually occurred at resolution.
            // No such Condition variant (ExchangeHappened / TargetWasDeclared) exists in the DSL.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::ExchangeControl {
                    target_a: EffectTarget::Source,
                    target_b: EffectTarget::DeclaredTarget { index: 0 },
                    duration: EffectDuration::Indefinite,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::UpToN {
                    count: 1,
                    inner: Box::new(TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                        controller: TargetController::Opponent,
                        ..Default::default()
                    })),
                }],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::partial(
            "'If you don't or can't make an exchange, sacrifice this creature.' — there is no \
             Condition variant expressing whether...",
        ),
        ..Default::default()
    }
}
