// Bloodsoaked Champion — {B}, Creature — Human Warrior 2/1
// This creature can't block.
// Raid — {1}{B}: Return this card from your graveyard to the battlefield.
// Activate only if you attacked this turn.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("bloodsoaked-champion"),
        name: "Bloodsoaked Champion".to_string(),
        mana_cost: Some(ManaCost { black: 1, ..Default::default() }),
        types: creature_types(&["Human", "Warrior"]),
        oracle_text: "This creature can't block.\nRaid — {1}{B}: Return this card from your graveyard to the battlefield. Activate only if you attacked this turn.".to_string(),
        power: Some(2),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::CantBlock),
            // Raid: {1}{B} from graveyard, activate only if you attacked this turn.
            // TODO: Condition::YouAttackedThisTurn does not exist in the DSL. The activation
            // condition "only if you attacked this turn" cannot be expressed. The ability is
            // omitted rather than implemented without the guard — producing wrong game state
            // (free return without raid condition) violates W5 policy.
        ],
        ..Default::default()
    }
}
