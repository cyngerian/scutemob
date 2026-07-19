// Siege-Gang Lieutenant — {3}{R}, Creature — Goblin 2/2
// Lieutenant — At the beginning of combat on your turn, if you control your commander,
//   create two 1/1 red Goblin creature tokens. Those tokens gain haste until end of turn.
// {2}, Sacrifice a Goblin: This creature deals 1 damage to any target.
//
// PB-OS9 / CR 903.3d: Lieutenant's condition half is now expressible --
// Condition::YouControlYourCommander is wired as an intervening-if on an
// AtBeginningOfCombat trigger (CR 603.4 resolution-time check). Token haste-until-EOT:
// permanent-haste TokenSpec.keywords is the accepted functionally-equivalent fallback
// (same rationale as loyal_apprentice.rs / DSL has no ApplyContinuousEffect scoped to
// "the permanents I just created").
//
// STILL BLOCKED (newly discovered during PB-OS9, verified by execution -- see
// loyal_apprentice.rs's top-of-file comment for the full account):
// `TriggerCondition::AtBeginningOfCombat` has no card-def sweep anywhere in the
// engine's turn-based-action machinery (`begin_combat()` only queues EMBLEM
// triggers). Confirmed empirically: the trigger never queues. Pre-existing engine
// gap, out of PB-OS9 scope to fix; also affects legion_warboss.rs,
// goblin_rabblemaster.rs, mirage_phalanx.rs, helm_of_the_host.rs. The Lieutenant DSL
// below is CR-correct and ready to fire once that sweep is added; the {2}, Sacrifice a
// Goblin activated ability is unaffected and fully functional.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("siege-gang-lieutenant"),
        name: "Siege-Gang Lieutenant".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            red: 1,
            ..Default::default()
        }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Lieutenant \u{2014} At the beginning of combat on your turn, if you control \
                      your commander, create two 1/1 red Goblin creature tokens. Those tokens \
                      gain haste until end of turn.\n{2}, Sacrifice a Goblin: This creature deals \
                      1 damage to any target."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // Lieutenant — CR 903.3d: "At the beginning of combat on your turn, if you
            // control your commander, create two 1/1 red Goblin creature tokens. Those
            // tokens gain haste until end of turn."
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
                        count: EffectAmount::Fixed(2),
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
                intervening_if: Some(Condition::YouControlYourCommander),
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // {2}, Sacrifice a Goblin: This creature deals 1 damage to any target.
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost {
                        generic: 2,
                        ..Default::default()
                    }),
                    Cost::Sacrifice(TargetFilter {
                        has_subtype: Some(SubType("Goblin".to_string())),
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }),
                ]),
                effect: Effect::DealDamage {
                    source: None,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    amount: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![TargetRequirement::TargetAny],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "PB-OS9: the Lieutenant CONDITION half is now correctly modeled -- intervening_if: \
             Condition::YouControlYourCommander on the AtBeginningOfCombat trigger (CR \
             903.3d/603.4). STILL BLOCKED: TriggerCondition::AtBeginningOfCombat has no card-def \
             sweep anywhere in the engine (begin_combat() only queues emblem triggers), confirmed \
             by execution -- the trigger never queues, so this ability is currently inert (not \
             wrong game state, just non-firing). Pre-existing engine gap, also affects \
             legion_warboss/goblin_rabblemaster/mirage_phalanx/helm_of_the_host. The {2}, \
             Sacrifice a Goblin activated ability IS fully modeled and functional.",
        ),
        ..Default::default()
    }
}
