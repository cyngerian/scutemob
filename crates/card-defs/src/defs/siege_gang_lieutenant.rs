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
// PB-RS3: the sweep gap is CLOSED -- `begin_combat` (turn_actions.rs) now scans the
// battlefield for card-def AtBeginningOfCombat triggers (in addition to the
// pre-existing emblem-only scan), so this ability fires. Flipped to Complete. The
// {2}, Sacrifice a Goblin activated ability is unaffected and remains fully functional.
//
// Accepted engine-wide limitation (F3, `memory/card-authoring/review-pb-rs3-roster.md`):
// `intervening_if` is checked only at resolution (resolution.rs:2125-2135), never at
// queue time, though CR 603.4 requires both. See loyal_apprentice.rs's top-of-file
// comment for the full account -- a pre-existing, engine-wide convention (documented
// at the upkeep sweep, turn_actions.rs:265-266), not a defect specific to this card.
// Filed as a seed (rider-seed-triage-2026-07-19.md) rather than blocking this flip.
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
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
