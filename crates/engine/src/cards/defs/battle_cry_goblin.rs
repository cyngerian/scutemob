// Battle Cry Goblin — {1}{R}, Creature — Goblin 2/2
// {1}{R}: Goblins you control get +1/+0 and gain haste until end of turn.
// Pack tactics — Whenever this creature attacks, if you attacked with creatures with total
// power 6 or greater this combat, create a 1/1 red Goblin creature token that's tapped
// and attacking.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("battle-cry-goblin"),
        name: "Battle Cry Goblin".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 1, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "{1}{R}: Goblins you control get +1/+0 and gain haste until end of turn.\nPack tactics \u{2014} Whenever this creature attacks, if you attacked with creatures with total power 6 or greater this combat, create a 1/1 red Goblin creature token that's tapped and attacking.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.4c / CR 613.1f: "{1}{R}: Goblins you control get +1/+0 and gain haste
            // until end of turn."
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost { generic: 1, red: 1, ..Default::default() }),
                effect: Effect::Sequence(vec![
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::PtModify,
                            modification: LayerModification::ModifyPower(1),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Goblin".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                    Effect::ApplyContinuousEffect {
                        effect_def: Box::new(ContinuousEffectDef {
                            layer: EffectLayer::Ability,
                            modification: LayerModification::AddKeyword(KeywordAbility::Haste),
                            filter: EffectFilter::CreaturesYouControlWithSubtype(
                                SubType("Goblin".to_string()),
                            ),
                            duration: EffectDuration::UntilEndOfTurn,
                            condition: None,
                        }),
                    },
                ]),
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            },
            // TODO: Pack tactics — "Whenever this creature attacks, if you attacked with
            // creatures with total power 6 or greater this combat, create a 1/1 red Goblin
            // creature token that's tapped and attacking."
            // Blocked: Condition::AttackedWithTotalPowerAtLeast(6) not in DSL.
        ],
        ..Default::default()
    }
}
