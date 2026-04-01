// Goblin King — {1}{R}{R}, Creature — Goblin 2/2
// Other Goblins get +1/+1 and have mountainwalk.
// NOTE: Goblin King's effect applies to ALL Goblins, not just yours. The DSL's
// OtherCreaturesYouControlWithSubtype only applies to your creatures — this is
// a known limitation. The +1/+1 uses AllCreaturesWithSubtype (all players) but
// "Other" is also approximated here. Using OtherCreaturesYouControlWithSubtype
// for both effects — correct for the controller, but missing opponent Goblins.
// TODO: DSL gap — no EffectFilter for "all other Goblins any player controls".
// AllCreaturesWithSubtype includes the source; OtherCreaturesYouControlWithSubtype
// excludes opponents' Goblins. Use AllCreaturesWithSubtype for mountainwalk grant
// (broadest correct approximation) and OtherCreaturesYouControlWithSubtype for +1/+1.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("goblin-king"),
        name: "Goblin King".to_string(),
        mana_cost: Some(ManaCost { generic: 1, red: 2, ..Default::default() }),
        types: creature_types(&["Goblin"]),
        oracle_text: "Other Goblins get +1/+1 and have mountainwalk. (They can't be blocked as long as defending player controls a Mountain.)".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // TODO: AllCreaturesWithSubtype includes Goblin King itself — "other" semantics
            // not expressible for cross-player tribal lords. This is the closest approximation.
            // +1/+1 to all Goblin creatures (any controller) — Layer 7c.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AllCreaturesWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // Mountainwalk to all Goblin creatures (any controller) — Layer 6.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Landwalk(
                        LandwalkType::BasicType(SubType("Mountain".to_string())),
                    )),
                    filter: EffectFilter::AllCreaturesWithSubtype(SubType("Goblin".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
        ],
        ..Default::default()
    }
}
