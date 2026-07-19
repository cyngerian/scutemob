// Goblin Rabblemaster — {2}{R}, Creature — Goblin Warrior 2/2
// Other Goblin creatures you control attack each combat if able.
// At the beginning of combat on your turn, create a 1/1 red Goblin creature token with haste.
// Whenever this creature attacks, it gets +1/+0 until end of turn for each other attacking Goblin.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-rabblemaster"),
        name: "Goblin Rabblemaster".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin", "Warrior"]),
        oracle_text: "Other Goblin creatures you control attack each combat if able.\nAt the \
                      beginning of combat on your turn, create a 1/1 red Goblin creature token \
                      with haste.\nWhenever Goblin Rabblemaster attacks, it gets +1/+0 until end \
                      of turn for each other attacking Goblin."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: "Other Goblins must attack" forced-attack restriction not in DSL.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Goblin".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Goblin".to_string())].into_iter().collect(),
                        colors: [Color::Red].into_iter().collect(),
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::Fixed(1),
                        supertypes: imbl::OrdSet::new(),
                        keywords: [KeywordAbility::Haste].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // CR 508.1m: "Whenever Goblin Rabblemaster attacks, it gets +1/+0 until end of
            // turn for each other attacking Goblin." Single (x1, no Sum) analogue of Goblin
            // Piledriver's x2 pump — see goblin_piledriver.rs for the shape rationale.
            // controller: EachPlayer (not Controller) is the CR-correct "each other attacking
            // Goblin" scope (any controller), matching the Commissar Severina Raine precedent;
            // identical to Controller-only in normal single-attacker combat, but EachPlayer is
            // the safe general reading. exclude_self excludes Rabblemaster itself via
            // ctx.source (WhenAttacks -> source is this creature).
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::ApplyContinuousEffect {
                    effect_def: Box::new(ContinuousEffectDef {
                        layer: EffectLayer::PtModify,
                        modification: LayerModification::ModifyPowerDynamic {
                            amount: Box::new(EffectAmount::AttackingCreatureCount {
                                controller: PlayerTarget::EachPlayer,
                                filter: Some(TargetFilter {
                                    has_subtype: Some(SubType("Goblin".to_string())),
                                    exclude_self: true,
                                    ..Default::default()
                                }),
                            }),
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
        completeness: Completeness::partial(
            "PB-OS5: the '+1/+0 for each other attacking Goblin' clause is now IMPLEMENTED \
             (AttackingCreatureCount{controller: EachPlayer, filter: has_subtype Goblin, \
             exclude_self} + ModifyPowerDynamic). Only the forced-attack clause remains blocked: \
             'Other Goblin creatures you control attack each combat if able' has no \
             GameRestriction (all existing variants are prohibitions; none is a subtype-filtered \
             must-attack requirement). Needs a new subtype-filtered must-attack GameRestriction \
             variant — out of PB-OS5 scope; tracked as its own seed. Combat token trigger is \
             correct.",
        ),
        ..Default::default()
    }
}
