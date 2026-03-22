// Patron of the Vein — {4}{B}{B} Creature — Vampire Shaman 4/4
// Flying
// When this creature enters, destroy target creature an opponent controls.
// Whenever a creature an opponent controls dies, exile it and put a +1/+1 counter on each Vampire you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("patron-of-the-vein"),
        name: "Patron of the Vein".to_string(),
        mana_cost: Some(ManaCost { generic: 4, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire", "Shaman"]),
        oracle_text:
            "Flying\nWhen this creature enters, destroy target creature an opponent controls.\nWhenever a creature an opponent controls dies, exile it and put a +1/+1 counter on each Vampire you control."
                .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 603.1: ETB trigger — destroy target creature an opponent controls.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    ..Default::default()
                })],
            },
            // TODO: "Whenever a creature an opponent controls dies, exile it and put a +1/+1
            // counter on each Vampire you control."
            // DSL gaps:
            // 1. TriggerCondition::WheneverCreatureDies is overbroad — it fires on any
            //    creature's death (including your own) with no controller filter available
            //    (KI-5). There is no WheneverCreatureAnOpponentControlsDies variant.
            // 2. "Exile it" requires targeting the specific dying creature (last-known
            //    information, now in the graveyard). ExileObject with a dying-creature
            //    reference is not in the DSL.
        ],
        ..Default::default()
    }
}
