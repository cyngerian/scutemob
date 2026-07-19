// Goblin Piledriver — {1}{R}, Creature — Goblin Warrior 1/2
// Protection from blue.
// Whenever this creature attacks, it gets +2/+0 until end of turn for each other
// attacking Goblin.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-piledriver"),
        name: "Goblin Piledriver".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Protection from blue.\nWhenever this creature attacks, it gets +2/+0 until \
                      end of turn for each other attacking Goblin."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![
            // CR 702.16: Protection from blue.
            AbilityDefinition::Keyword(KeywordAbility::ProtectionFrom(
                ProtectionQuality::FromColor(Color::Blue),
            )),
            // CR 508.1m: "Whenever this creature attacks, it gets +2/+0 until end of turn for
            // each other attacking Goblin." "+2/+0 per" = 2 x count, expressed as
            // Sum(count, count) rather than a new EffectAmount multiplier primitive (PB-OS5
            // design decision — single consumer, YAGNI). Both summands are
            // AttackingCreatureCount{controller: EachPlayer, filter: has_subtype Goblin +
            // exclude_self}, which excludes Piledriver itself via ctx.source (WhenAttacks ->
            // source is this creature) and counts attacking Goblins of ANY controller
            // (Piledriver ruling 2004-10-04: counted at resolution).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::Sum(
                                Box::new(EffectAmount::AttackingCreatureCount {
                                    controller: PlayerTarget::EachPlayer,
                                    filter: Some(TargetFilter {
                                        has_subtype: Some(SubType("Goblin".to_string())),
                                        exclude_self: true,
                                        ..Default::default()
                                    }),
                                }),
                                Box::new(EffectAmount::AttackingCreatureCount {
                                    controller: PlayerTarget::EachPlayer,
                                    filter: Some(TargetFilter {
                                        has_subtype: Some(SubType("Goblin".to_string())),
                                        exclude_self: true,
                                        ..Default::default()
                                    }),
                                }),
                            )),
                            negate: false,
                        },
                        filter: EffectFilter::Source,
                        duration: EffectDuration::UntilEndOfTurn,
                        condition: None,
                    }),
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
        ],
        ..Default::default()
    }
}
