// Nether Traitor — {B}{B}, Creature — Spirit 1/1; Haste, Shadow.
// Whenever another creature is put into your graveyard from the battlefield, you may pay
// {B}. If you do, return this card from your graveyard to the battlefield.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nether-traitor"),
        name: "Nether Traitor".to_string(),
        mana_cost: Some(ManaCost {
            black: 2,
            ..Default::default()
        }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Haste\nShadow (This creature can block or be blocked by only creatures with \
                      shadow.)\nWhenever another creature is put into your graveyard from the \
                      battlefield, you may pay {B}. If you do, return this card from your \
                      graveyard to the battlefield."
            .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Shadow),
            // CR 603.3 (TriggerZone::Graveyard) / CR 118.12: "Whenever another creature is put
            // into your graveyard from the battlefield, you may pay {B}. If you do, return this
            // card from your graveyard to the battlefield."
            AbilityDefinition::Triggered {
                once_per_turn: false,
                // Oracle "put into YOUR graveyard" is an ownership condition (CR 404.3), but the
                // DSL has no owner-scoped death trigger, so this keys on controller = You. The two
                // diverge only under gain-control (a creature you OWN but an opponent controls dies
                // to your graveyard — oracle fires, this doesn't; and vice-versa). Best available
                // approximation; matches the corpus convention (athreos, fecundity).
                trigger_condition: TriggerCondition::WheneverCreatureDies {
                    controller: Some(TargetController::You),
                    exclude_self: true,
                    nontoken_only: false,
                    filter: None,
                },
                effect: Effect::MayPayThenEffect {
                    cost: Cost::Mana(ManaCost {
                        black: 1,
                        ..Default::default()
                    }),
                    payer: PlayerTarget::Controller,
                    then: Box::new(Effect::MoveZone {
                        target: EffectTarget::Source,
                        to: ZoneTarget::Battlefield { tapped: false },
                        controller_override: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: Some(TriggerZone::Graveyard),
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
