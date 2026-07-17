// Tiamat — {2}{W}{U}{B}{R}{G}, Legendary Creature — Dragon God 7/7
// Flying
// When Tiamat enters, if you cast it, search your library for up to five Dragon cards
// not named Tiamat that each have different names, reveal them, put them into your hand,
// then shuffle.
//
// Flying is implemented.
//
// TODO: DSL gap — the ETB triggered ability requires SearchLibrary with:
// 1. "up to five" (variable count) cards matching a subtype filter (Dragon)
// 2. An exclusion filter ("not named Tiamat")
// 3. A "each have different names" uniqueness constraint
// 4. Destination: hand (not battlefield)
// SearchLibrary in the DSL only supports basic_land_filter() targeting battlefield.
// This ability is omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("tiamat"),
        name: "Tiamat".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            blue: 1,
            black: 1,
            red: 1,
            green: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon", "God"],
        ),
        oracle_text: "Flying\nWhen Tiamat enters, if you cast it, search your library for up to five Dragon cards not named Tiamat that each have different names, reveal them, put them into your hand, then shuffle.".to_string(),
        power: Some(7),
        toughness: Some(7),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
        ],
        completeness: Completeness::partial("Blocked on SearchLibrary having no count field (card_definition.rs:1648) — 'up to five' — and no 'each have different names' uniqueness constraint. Also needs a name-exclusion (TargetFilter.has_name is inclusion-only). NOT gaps: arbitrary TargetFilter and ZoneTarget::Hand destinations are supported (see thaumatic_compass.rs, tooth_and_nail.rs), and 'if you cast it' is Condition::WasCast. Flying implemented."),
        ..Default::default()
    }
}
