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
            // Other Elves you control get +1/+1.
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Elf".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // ENGINE-BLOCKED: "Whenever you cast an Elf spell, you may pay {G}. If you
            // do, draw a card." PB-AC2's Effect::MayPayThenEffect (CR 118.12) now covers
            // the "may pay {G} -> draw" rider, but the trigger condition itself is still
            // blocked: TriggerCondition::WheneverYouCastSpell only supports
            // spell_type_filter (Vec<CardType>) and chosen_subtype_filter (dynamic,
            // reads ctx.chosen_creature_type) — there is no fixed-subtype ("Elf")
            // filter field. Casting an Elf spell cannot be distinguished from casting
            // any other creature spell. Genuine remaining gap (PB-AC7 territory per
            // pb-plan-AC2.md); per W5 policy, omitted rather than firing on every
            // creature spell.
        ],
        ..Default::default()
    }
}
