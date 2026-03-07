// Silverblade Paladin — {1}{W}{W}, Creature — Human Knight 2/2; Soulbond, grants double strike.
// CR 702.93: Soulbond pairs two creatures; both receive granted abilities while paired.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silverblade-paladin"),
        name: "Silverblade Paladin".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, ..Default::default() }),
        types: creature_types(&["Human", "Knight"]),
        oracle_text: "Soulbond (You may pair this creature with another unpaired creature when either enters. They remain paired for as long as you control both of them.)\nAs long as this creature is paired with another creature, both creatures have double strike.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Soulbond {
                grants: vec![SoulbondGrant {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::DoubleStrike),
                }],
            },
        ],
        ..Default::default()
    }
}
