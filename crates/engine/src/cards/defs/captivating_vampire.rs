// Captivating Vampire — {1}{B}{B}, Creature — Vampire 2/2
// Other Vampire creatures you control get +1/+1.
// Tap five untapped Vampires you control: Gain control of target creature.
// It becomes a Vampire in addition to its other types.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("captivating-vampire"),
        name: "Captivating Vampire".to_string(),
        mana_cost: Some(ManaCost { generic: 1, black: 2, ..Default::default() }),
        types: creature_types(&["Vampire"]),
        oracle_text: "Other Vampire creatures you control get +1/+1.\nTap five untapped Vampires you control: Gain control of target creature. It becomes a Vampire in addition to its other types.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            // CR 613.1c / Layer 7c: "Other Vampire creatures you control get +1/+1."
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::OtherCreaturesYouControlWithSubtype(SubType("Vampire".to_string())),
                    duration: EffectDuration::WhileSourceOnBattlefield,
                },
            },
            // TODO: DSL gap — "Tap five untapped Vampires you control: Gain control of target
            // creature. It becomes a Vampire in addition to its other types."
            // Cost::TapNCreaturesWithSubtype(5, SubType) does not exist.
            // Also needs: gain control (SetController) + add subtype (AddSubtype).
        ],
        ..Default::default()
    }
}
