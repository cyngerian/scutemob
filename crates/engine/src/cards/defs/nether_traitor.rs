// Nether Traitor — {B}{B}, Creature — Spirit 1/1; Haste, Shadow.
// Triggered ability returns this card from graveyard when another creature dies —
// TODO: DSL gap — triggered ability from graveyard zone not expressible, and
// mana-payment conditional ("you may pay {B}") not supported by TriggerCondition.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("nether-traitor"),
        name: "Nether Traitor".to_string(),
        mana_cost: Some(ManaCost { black: 2, ..Default::default() }),
        types: creature_types(&["Spirit"]),
        oracle_text: "Haste\nShadow (This creature can block or be blocked by only creatures with shadow.)\nWhenever another creature is put into your graveyard from the battlefield, you may pay {B}. If you do, return this card from your graveyard to the battlefield.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Shadow),
        ],
        // TODO: triggered ability from graveyard ("may pay {B}, return from graveyard")
        ..Default::default()
    }
}
