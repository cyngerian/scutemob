// Kiki-Jiki, Mirror Breaker — {2}{R}{R}{R}, Legendary Creature — Goblin Shaman 2/2
// Haste
// {T}: Create a token that's a copy of target nonlegendary creature you control, except
// it has haste. Sacrifice it at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kiki-jiki-mirror-breaker"),
        name: "Kiki-Jiki, Mirror Breaker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Shaman"],
        ),
        oracle_text: "Haste\n{T}: Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // {T}: Create a token that's a copy of target nonlegendary creature you control,
            // except it has haste. Sacrifice it at the beginning of the next end step.
            // TODO: target filter lacks "nonlegendary" restriction (TargetFilter has no nonlegendary bool).
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::DeclaredTarget { index: 0 },
                    enters_tapped_and_attacking: false,
                    except_not_legendary: false,
                    gains_haste: true,
                    delayed_action: Some((
                        crate::state::stubs::DelayedTriggerTiming::AtNextEndStep,
                        crate::state::stubs::DelayedTriggerAction::SacrificeObject,
                    )),
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
            },
        ],
        ..Default::default()
    }
}
