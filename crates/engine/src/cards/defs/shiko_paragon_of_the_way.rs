// Shiko, Paragon of the Way — {2}{U}{R}{W} Legendary Creature — Spirit Dragon 4/5
// Flying, vigilance
// When Shiko enters, exile target nonland card with mana value 3 or less from your
// graveyard. Copy it, then you may cast the copy without paying its mana cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("shiko-paragon-of-the-way"),
        name: "Shiko, Paragon of the Way".to_string(),
        mana_cost: Some(ManaCost { generic: 2, blue: 1, red: 1, white: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Spirit", "Dragon"],
        ),
        oracle_text: "Flying, vigilance\nWhen Shiko enters, exile target nonland card with mana value 3 or less from your graveyard. Copy it, then you may cast the copy without paying its mana cost. (A copy of a permanent spell becomes a token.)".to_string(),
        power: Some(4),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            AbilityDefinition::Keyword(KeywordAbility::Vigilance),
            // ETB: exile target nonland card MV<=3 from your graveyard, copy + free cast.
            // TODO: Copy-and-cast-from-exile pattern not in DSL. Exile-only would produce
            // wrong game state (removes card from graveyard without providing the free cast
            // benefit), so the full triggered ability is left as a TODO.
            // DSL gap: Effect::CopyAndCastFromExile or PlayExiledCard with copy semantics.
        ],
        ..Default::default()
    }
}
