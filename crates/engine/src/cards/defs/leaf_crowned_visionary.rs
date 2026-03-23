// Leaf-Crowned Visionary — {G}{G}, Creature — Elf Druid 1/1
// Other Elves you control get +1/+1.
// Whenever you cast an Elf spell, you may pay {G}. If you do, draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("leaf-crowned-visionary"),
        name: "Leaf-Crowned Visionary".to_string(),
        mana_cost: Some(ManaCost { green: 2, ..Default::default() }),
        types: creature_types(&["Elf", "Druid"]),
        oracle_text: "Other Elves you control get +1/+1.\nWhenever you cast an Elf spell, you may pay {G}. If you do, draw a card.".to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // Other Elves you control get +1/+1
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: "Whenever you cast an Elf spell, may pay {G}" — WheneverYouCastSpell
            //   lacks spell-type filter + optional payment. Removed to avoid wrong game state.
        ],
        ..Default::default()
    }
}
