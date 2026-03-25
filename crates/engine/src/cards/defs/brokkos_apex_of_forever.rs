// Brokkos, Apex of Forever — {2}{U/B}{G}{G}, Legendary Creature — Nightmare Beast Elemental 6/6
// Mutate {3}{U/B}{G}
// Trample
// You may cast Brokkos, Apex of Forever from your graveyard using its mutate ability.
//
// CR 702.140a: Mutate is an alternative cost targeting a non-Human creature you own.
// TODO: "You may cast this card from your graveyard using its mutate ability" requires a
//       cast-from-zone permission system that does not yet exist. This ability is omitted.
//       When this is implemented, casting.rs must allow AltCostKind::Mutate from the graveyard
//       when the source card has this ability, and the card must be found via find_in_graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("brokkos-apex-of-forever"),
        name: "Brokkos, Apex of Forever".to_string(),
        // Main cost {2}{B}{G}{U} — three separate colored pips, no hybrid.
        // The hybrid {U/B} only appears in the mutate cost below (CR 702.140a).
        mana_cost: Some(ManaCost {
            generic: 2,
            blue: 1,
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Nightmare", "Beast", "Elemental"],
        ),
        oracle_text:
            "Mutate {3}{U/B}{G} (If you cast this spell for its mutate cost, put it over or under target non-Human creature you own. They mutate into the creature on top plus all abilities from under it.)\nTrample\nYou may cast Brokkos, Apex of Forever from your graveyard using its mutate ability."
                .to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            // CR 702.140a: Mutate keyword marker for presence-checking.
            AbilityDefinition::Keyword(KeywordAbility::Mutate),
            // CR 702.140a: Mutate cost {3}{U/B}{G}
            AbilityDefinition::MutateCost {
                cost: ManaCost {
                    generic: 3,
                    green: 1,
                    hybrid: vec![HybridMana::ColorColor(ManaColor::Blue, ManaColor::Black)],
                    ..Default::default()
                },
            },
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: "You may cast this card from your graveyard using its mutate ability."
            // This requires a cast-from-zone permission system (deferred). No effect stub
            // needed — the casting.rs validation would need to allow graveyard-zone casting
            // for this specific ability. Omitted until that infrastructure exists.
        ],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        activated_ability_cost_reductions: vec![],
    }
}
