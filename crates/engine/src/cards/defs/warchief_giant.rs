// 110. Warchief Giant — {3}{R}{R}, Creature — Giant Warrior 5/3; Haste, Myriad.
// CR 702.116a: Myriad — whenever this attacks, create a token copy tapped and
// attacking each other opponent. Exile tokens at end of combat.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("warchief-giant"),
        name: "Warchief Giant".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 2, ..Default::default() }),
        types: creature_types(&["Giant", "Warrior"]),
        oracle_text: "Haste\nMyriad (Whenever this creature attacks, for each opponent other than the defending player, you may create a token that's a copy of this creature that's tapped and attacking that player. If one or more tokens are created this way, exile the tokens at end of combat.)".to_string(),
        power: Some(5),
        toughness: Some(3),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            AbilityDefinition::Keyword(KeywordAbility::Myriad),
        ],
        back_face: None,
    }
}
