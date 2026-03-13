// Teneb, the Harvester — {3}{W}{B}{G}, Legendary Creature — Dragon 6/6
// Flying
// Whenever Teneb deals combat damage to a player, you may pay {2}{B}. If you do,
// put target creature card from a graveyard onto the battlefield under your control.
// TODO: DSL gap — "pay {2}{B} to return target creature card from any graveyard to battlefield"
// is a triggered ability with an optional mana payment and a targeted graveyard-to-battlefield
// effect; no return_from_graveyard DSL effect exists.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("teneb-the-harvester"),
        name: "Teneb, the Harvester".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 1, black: 1, green: 1, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Dragon"],
        ),
        oracle_text: "Flying\nWhenever Teneb deals combat damage to a player, you may pay {2}{B}. If you do, put target creature card from a graveyard onto the battlefield under your control.".to_string(),
        power: Some(6),
        toughness: Some(6),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // TODO: triggered — whenever Teneb deals combat damage to a player, may pay {2}{B}
            // to return target creature card from any graveyard to battlefield under your control.
            // DSL gap: no return_from_graveyard effect; no optional mana payment trigger pattern.
        ],
        ..Default::default()
    }
}
