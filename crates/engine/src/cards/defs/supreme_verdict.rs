// 28. Supreme Verdict — {1WWU}, Sorcery; destroy all creatures. It can't be
// countered.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("supreme-verdict"),
        name: "Supreme Verdict".to_string(),
        mana_cost: Some(ManaCost { white: 2, blue: 1, generic: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "This spell can't be countered.\nDestroy all creatures.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::ForEach {
                over: ForEachTarget::EachCreature,
                effect: Box::new(Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                }),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: true,
        }],
        ..Default::default()
    }
}
