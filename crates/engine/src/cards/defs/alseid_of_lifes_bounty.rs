// Alseid of Life's Bounty — {W}, Enchantment Creature — Nymph 1/1
// Lifelink
// {1}, Sacrifice this creature: Target creature or enchantment you control gains
// protection from the color of your choice until end of turn.
//
// Lifelink is implemented.
//
// TODO: DSL gap — the activated ability requires "protection from the color of your
// choice", which is a player-interactive color selection at activation time.
// ProtectionFrom(ProtectionQuality::FromColor(...)) can represent the result, but
// there is no DSL mechanism to prompt the player for a color choice and then apply
// it as a temporary continuous effect on the target.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("alseid-of-lifes-bounty"),
        name: "Alseid of Life's Bounty".to_string(),
        mana_cost: Some(ManaCost {
            white: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Enchantment, CardType::Creature], &["Nymph"]),
        oracle_text: "Lifelink\n{1}, Sacrifice this creature: Target creature or enchantment you control gains protection from the color of your choice until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Lifelink),
        ],
        ..Default::default()
    }
}
