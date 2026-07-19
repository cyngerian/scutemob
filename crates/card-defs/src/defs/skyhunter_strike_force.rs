// Skyhunter Strike Force — {2}{W}, Creature — Cat Knight 2/2
// Flying
// Melee (Whenever this creature attacks, it gets +1/+1 until end of turn for each
// opponent you attacked this combat.)
// Lieutenant — As long as you control your commander, other creatures you control
// have melee.
//
// PB-OS9 / CR 903.3d: Lieutenant anthem is a conditional Layer 6 grant of Melee to
// other creatures you control, gated on Condition::YouControlYourCommander. Post-
// PB-EF3b the granted Melee synthesizes its attack trigger, so this anthem now
// actually fires for the recipient creatures.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("skyhunter-strike-force"),
        name: "Skyhunter Strike Force".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 1,
            ..Default::default()
        }),
        types: creature_types(&["Cat", "Knight"]),
        oracle_text: "Flying\nMelee (Whenever this creature attacks, it gets +1/+1 until end of \
                      turn for each opponent you attacked this combat.)\nLieutenant — As long as \
                      you control your commander, other creatures you control have melee."
            .to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 702.121a: printed Melee (self).
            AbilityDefinition::Keyword(KeywordAbility::Melee),
            // Lieutenant — CR 903.3d: "As long as you control your commander, other
            // creatures you control have melee." Layer 6 conditional grant.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Melee),
                    filter: EffectFilter::OtherCreaturesYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::YouControlYourCommander),
                },
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
