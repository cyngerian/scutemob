// Sword of the Animist — {2}, Legendary Artifact — Equipment
// Equipped creature gets +1/+1.
// Whenever equipped creature attacks, you may search your library for a basic land card,
// put it onto the battlefield tapped, then shuffle.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("sword-of-the-animist"),
        name: "Sword of the Animist".to_string(),
        mana_cost: Some(ManaCost { generic: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/+1.\nWhenever equipped creature attacks, you may search your library for a basic land card, put it onto the battlefield tapped, then shuffle.\nEquip {2}".to_string(),
        abilities: vec![
            // Equipped creature gets +1/+1 (layer 7c).
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyPower(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyToughness(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — "Whenever equipped creature attacks" trigger condition
            // (WhenEquippedCreatureAttacks) does not exist. WhenAttacks is self-only.
            // Equip {2}
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        ..Default::default()
    }
}
