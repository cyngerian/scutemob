// Scion of Draco — {12}, Artifact Creature — Dragon 4/4
// Domain — This spell costs {2} less to cast for each basic land type among lands you control.
// Flying
// Each creature you control has vigilance if it's white, hexproof if it's blue, lifelink if
// it's black, first strike if it's red, and trample if it's green.
use crate::cards::helpers::*;

fn color_keyword_grant(color: Color, keyword: KeywordAbility) -> AbilityDefinition {
    AbilityDefinition::Static {
        continuous_effect: ContinuousEffectDef {
            layer: EffectLayer::Ability,
            modification: LayerModification::AddKeyword(keyword),
            filter: EffectFilter::CreaturesYouControlWithColor(color),
            duration: EffectDuration::WhileSourceOnBattlefield,
            condition: None,
        },
    }
}

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("scion-of-draco"),
        name: "Scion of Draco".to_string(),
        mana_cost: Some(ManaCost {
            generic: 12,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact, CardType::Creature], &["Dragon"]),
        oracle_text: "Domain — This spell costs {2} less to cast for each basic land type among \
                      lands you control.\nFlying\nEach creature you control has vigilance if it's \
                      white, hexproof if it's blue, lifelink if it's black, first strike if it's \
                      red, and trample if it's green."
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            color_keyword_grant(Color::White, KeywordAbility::Vigilance),
            color_keyword_grant(Color::Blue, KeywordAbility::Hexproof),
            color_keyword_grant(Color::Black, KeywordAbility::Lifelink),
            color_keyword_grant(Color::Red, KeywordAbility::FirstStrike),
            color_keyword_grant(Color::Green, KeywordAbility::Trample),
        ],
        self_cost_reduction: Some(SelfCostReduction::BasicLandTypes { per: 2 }),
        ..Default::default()
    }
}
