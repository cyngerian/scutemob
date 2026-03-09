// Camellia, the Seedmiser — {1}{B}{G} Legendary Creature — Squirrel Warlock 3/3
// Menace; other Squirrels get menace (TODO: continuous grant — needs exclude-source filter);
// sacrifice-Food trigger creates Squirrel token (TODO: TriggerCondition::WhenSacrificeFood);
// {2}, Forage ability: TODO — Cost enum has no Forage variant; DSL gap documented below.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("camellia-the-seedmiser"),
        name: "Camellia, the Seedmiser".to_string(),
        mana_cost: Some(ManaCost { black: 1, green: 1, generic: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Squirrel", "Warlock"],
        ),
        oracle_text: "Menace\nOther Squirrels you control have menace.\nWhenever you sacrifice one or more Foods, create a 1/1 green Squirrel creature token.\n{2}, Forage: Put a +1/+1 counter on each other Squirrel you control. (To forage, exile three cards from your graveyard or sacrifice a Food.)".to_string(),
        abilities: vec![
            // CR 702.110: Menace (self).
            AbilityDefinition::Keyword(KeywordAbility::Menace),
            // TODO: "Other Squirrels you control have menace." — requires a continuous grant
            // ability with subtype filter (has_subtype: Squirrel, controller: You, exclude_self).
            // DSL gap: TargetFilter has no exclude_source field. Deferred.

            // TODO: "Whenever you sacrifice one or more Foods, create a 1/1 green Squirrel token."
            // Requires TriggerCondition::WheneverYouSacrificeFood (not yet implemented).
            // Deferred until sacrifice-trigger infrastructure is added.

            // TODO: "{2}, Forage: Put a +1/+1 counter on each other Squirrel you control."
            // The Forage cost mechanic is implemented in ActivationCost (game_object.rs) and
            // ActivatedAbility, but CardDefinition.abilities uses AbilityDefinition::Activated
            // with Cost enum which has no Forage variant. To express Forage on a card definition,
            // either add Cost::Forage or add AbilityDefinition::ForageActivated { mana_cost, effect }.
            // The engine-side forage payment is tested via crates/engine/tests/forage.rs (7 tests
            // using game states constructed directly with ActivationCost { forage: true }).
            // Deferred until Cost enum gains a Forage variant.
        ],
        power: Some(3),
        toughness: Some(3),
        back_face: None,
    }
}
