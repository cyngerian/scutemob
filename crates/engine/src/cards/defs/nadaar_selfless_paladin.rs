// Nadaar, Selfless Paladin — {3}{W}, Legendary Creature — Dragon Knight 3/3
// Vigilance
// Whenever Nadaar, Selfless Paladin enters the battlefield or attacks, venture into the dungeon.
// Other creatures you control get +1/+1 as long as you've completed a dungeon.
//
// CR 702.20a: Vigilance — doesn't tap when attacking.
// CR 701.49a-c: Venture into the dungeon.
// CR 309.7: "as long as you've completed a dungeon" checked via CompletedADungeon condition.
//
// DSL gap: "Other creatures you control get +1/+1 as long as you've completed a dungeon"
// requires EffectFilter::OtherCreaturesControlledBy (excludes self, scoped to controller).
// The AbilityDefinition::Static has no condition field for "as long as" clauses.
// Both gaps are deferred; tracked alongside Ultramarines Honour Guard (same DSL gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nadaar-selfless-paladin"),
        name: "Nadaar, Selfless Paladin".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "Knight"],
        ),
        oracle_text: "Vigilance\nWhenever Nadaar, Selfless Paladin enters the battlefield or attacks, venture into the dungeon.\nOther creatures you control get +1/+1 as long as you've completed a dungeon."
            .to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            // CR 702.20a: Vigilance — this creature doesn't tap when it attacks.
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // CR 701.49a-c: ETB trigger — venture into the dungeon when Nadaar enters.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenEntersBattlefield,
                effect: Effect::VentureIntoDungeon,
                intervening_if: None,
                targets: vec![],
            },
            // CR 701.49a-c: Attack trigger — venture into the dungeon when Nadaar attacks.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenAttacks,
                effect: Effect::VentureIntoDungeon,
                intervening_if: None,
                targets: vec![],
            },
            // CR 309.7 / CR 613.1c: "Other creatures you control get +1/+1 as long as
            // you've completed a dungeon."
            // TODO: AbilityDefinition::Static lacks a condition field for "as long as" clauses.
            // The OtherCreaturesYouControl filter is now available, but the dungeon-completion
            // condition is still needed. Unconditionally granting +1/+1 would be incorrect.
            // Deferred until condition-on-static is implemented.
        ],
        color_indicator: None,
        back_face: None,
    }
}
