// Nethroi, Apex of Death — {2}{W}{B}{G}, Legendary Creature — Cat Nightmare Beast 5/5
// Mutate {4}{G}{B}{B} (hybrid {G/W} simplified to {G} — hybrid mana is a DSL gap)
// Lifelink, Deathtouch
// Whenever this creature mutates, return any number of target creature cards with total power
// 10 or less from your graveyard to the battlefield.
//
// CR 702.140a: Mutate is an alternative cost targeting a non-Human creature you own.
// CR 702.140d: "Whenever this creature mutates" fires after a successful merge.
// TODO: The "total power 10 or less" multi-target graveyard constraint is a DSL gap.
//       Effect::ReturnFromGraveyard does not exist. The trigger is stubbed with no effect.
// TODO: Hybrid mana {G/W} in mutate cost simplified to {G}. Full hybrid support is deferred.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nethroi-apex-of-death"),
        name: "Nethroi, Apex of Death".to_string(),
        mana_cost: Some(ManaCost { generic: 2, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Cat", "Nightmare", "Beast"],
        ),
        oracle_text:
            "Mutate {4}{G/W}{B}{B} (If you cast this spell for its mutate cost, put it over or under target non-Human creature you own. They mutate into the creature on top plus all abilities from under it.)\nLifelink\nDeathtouch\nWhenever this creature mutates, return any number of target creature cards with total power 10 or less from your graveyard to the battlefield."
                .to_string(),
        power: Some(5),
        toughness: Some(5),
        abilities: vec![
            // CR 702.140a: Mutate keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            // CR 702.140a: Mutate cost {4}{G}{B}{B} (hybrid {G/W} simplified to {G}).
            // TODO: hybrid mana {G/W} not yet supported; using {G} as approximation.
            AbilityDefinition::MutateCost {
                cost: ManaCost { generic: 4, green: 1, black: 2, ..Default::default() },
            },
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
            AbilityDefinition::Keyword(KeywordAbility::Deathtouch),
            // CR 702.140d: "Whenever this creature mutates, return creature cards from graveyard."
            // TODO: Effect::ReturnFromGraveyard with total-power constraint does not exist (DSL gap).
            //       This trigger stub fires but has no executable effect at resolution.
            AbilityDefinition::Triggered {
                trigger_condition: TriggerCondition::WhenMutates,
                effect: Effect::Nothing,
                intervening_if: None,
                targets: vec![],
            },
        ],
        color_indicator: None,
        back_face: None,
    }
}
