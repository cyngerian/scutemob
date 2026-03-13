// Gilded Drake — {1}{U}, Creature — Drake 3/3
// Flying; ETB: exchange control with up to one target opponent's creature (or sacrifice self)
// TODO: ETB exchange-control targeted trigger not in DSL
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gilded-drake"),
        name: "Gilded Drake".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            blue: 1,
            ..Default::default()
        }),
        types: creature_types(&["Drake"]),
        oracle_text: "Flying\nWhen this creature enters, exchange control of this creature and up to one target creature an opponent controls. If you don't or can't make an exchange, sacrifice this creature. This ability still resolves if its target becomes illegal.".to_string(),
        power: Some(3),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: ETB trigger to exchange control of this creature with a targeted opponent
            // creature (with conditional sacrifice if no exchange) — exchange control and
            // conditional sacrifice effects not in DSL (targeted_trigger gap).
        ],
        ..Default::default()
    }
}
