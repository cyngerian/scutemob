// Wingcrafter — {U}, Creature — Human Wizard 1/1
// Soulbond; as long as paired, both creatures have flying.
// CR 702.95a: Soulbond pairing; both receive Flying while paired.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("wingcrafter"),
        name: "Wingcrafter".to_string(),
        mana_cost: Some(ManaCost { blue: 1, ..Default::default() }),
        types: creature_types(&["Human", "Wizard"]),
        oracle_text: "Soulbond (You may pair this creature with another unpaired creature when either enters. They remain paired for as long as you control both of them.)\nAs long as this creature is paired with another creature, both creatures have flying.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            AbilityDefinition::Soulbond {
                grants: vec![SoulbondGrant {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Flying),
                }],
            },
        ],
        ..Default::default()
    }
}
