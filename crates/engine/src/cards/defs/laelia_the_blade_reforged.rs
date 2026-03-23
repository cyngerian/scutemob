// Laelia, the Blade Reforged — {2}{R}, Legendary Creature — Spirit Warrior 2/2
// Haste
// Whenever Laelia attacks, exile the top card of your library. You may play that card this turn.
// Whenever one or more cards are put into exile from your library and/or your graveyard,
// put a +1/+1 counter on Laelia.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("laelia-the-blade-reforged"),
        name: "Laelia, the Blade Reforged".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Spirit", "Warrior"],
        ),
        oracle_text: "Haste\nWhenever Laelia attacks, exile the top card of your library. You may play that card this turn.\nWhenever one or more cards are put into exile from your library and/or your graveyard, put a +1/+1 counter on Laelia.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: DSL gap — attack trigger that exiles top card + grants play permission.
            // Effect::PlayExiledCard exists but "exile top card of library" + "this turn"
            // permission grant is not in DSL.
            // TODO: DSL gap — "Whenever cards are put into exile from library/graveyard"
            // trigger condition does not exist.
        ],
        ..Default::default()
    }
}
