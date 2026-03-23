// Derevi, Empyrial Tactician — {G}{W}{U}, Legendary Creature — Bird Wizard 2/3
// Flying
// When Derevi enters and whenever a creature you control deals combat damage to a player,
// you may tap or untap target permanent.
// {1}{G}{W}{U}: Put Derevi onto the battlefield from the command zone.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("derevi-empyrial-tactician"),
        name: "Derevi, Empyrial Tactician".to_string(),
        mana_cost: Some(ManaCost { green: 1, white: 1, blue: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Bird", "Wizard"]),
        oracle_text: "Flying\nWhen Derevi enters and whenever a creature you control deals combat damage to a player, you may tap or untap target permanent.\n{1}{G}{W}{U}: Put Derevi onto the battlefield from the command zone.".to_string(),
        power: Some(2),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: DSL gap — "When Derevi enters and whenever a creature you control deals
            // combat damage to a player" is a dual-trigger (ETB + per-creature combat damage)
            // with tap/untap target permanent choice. Neither trigger condition exists.
            // TODO: DSL gap — "{1}{G}{W}{U}: Put Derevi onto the battlefield from the command
            // zone" is a special activated ability from the command zone that bypasses casting.
            // No ActivateFromCommandZone mechanism exists.
        ],
        ..Default::default()
    }
}
