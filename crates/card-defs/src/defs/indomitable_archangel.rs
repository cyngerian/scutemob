// Indomitable Archangel — {2}{W}{W}, Creature — Angel 4/4
// Flying
// Metalcraft — Artifacts you control have shroud as long as you control three or more artifacts.
//
// CR 207.2c / CR 604.2 (Metalcraft): **there is no CR 702.x entry for Metalcraft.** It is
// an ABILITY WORD -- CR 207.2c names it in its own list and says ability words "have no
// special rules meaning and no individual entries in the Comprehensive Rules" -- so the
// clause is an ordinary CR 604.2 conditional static ability and nothing more. This line
// cited **CR 702.45a**, which is **Bushido**, from this def's authoring until PB-DX42b
// (`scutemob-233`, 2026-09-05) checked it against the rules server. Corrected rather than
// swept, because this def is PB-DX42b's own headline card and a wrong cite on the card a
// batch is about is the shape PB-DX27 exists to catch. `OOS-DX42b-2`.
//
// CR 613.1d: the condition reads CARD TYPES, which are set in Layer 4 -- strictly EARLIER
// than this effect's own Layer 6, which is exactly why PB-DX42b's layer-bounded query can
// answer it without re-entering itself, and why a Blinkmoth Nexus animated into an
// artifact creature now feeds this count as CR 613.1d requires.
// CR 613.1f (Layer 6): Static ability — grants shroud to all artifacts you control.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("indomitable-archangel"),
        name: "Indomitable Archangel".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        }),
        types: creature_types(&["Angel"]),
        oracle_text: "Flying\nMetalcraft \u{2014} Artifacts you control have shroud as long as \
                      you control three or more artifacts. (An artifact with shroud can't be the \
                      target of spells or abilities.)"
            .to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Flying),
            // CR 613.1f (Layer 6): "Artifacts you control have shroud as long as you
            // control three or more artifacts." (Metalcraft condition)
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::Ability,
                    modification: LayerModification::AddKeyword(KeywordAbility::Shroud),
                    filter: EffectFilter::ArtifactsYouControl,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: Some(Condition::YouControlNOrMoreWithFilter {
                        count: 3,
                        filter: TargetFilter {
                            has_card_type: Some(CardType::Artifact),
                            ..Default::default()
                        },
                    }),
                },
            },
        ],
        ..Default::default()
    }
}
