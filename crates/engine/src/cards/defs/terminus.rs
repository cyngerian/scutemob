// 72. Terminus — {4}{W}{W}, Sorcery; Put all creatures on the bottom of their owners'
// libraries. Miracle {W} (You may cast this card for its miracle cost. Cast it only
// as the first card you drew this turn.)
//
// Effect approximation: destroys all creatures (engine does not yet support
// "put on bottom of owner's library" as a ForEach with per-creature owner
// routing). Terminus is included primarily to validate the Miracle keyword
// (CR 702.94). The full oracle text and mana cost are correct.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("terminus"),
        name: "Terminus".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Put all creatures on the bottom of their owners' libraries.\nMiracle {W} (You may cast this card for its miracle cost. Cast it only as the first card you drew this turn.)".to_string(),
        abilities: vec![
            // CR 702.94a: Miracle keyword marker.
            AbilityDefinition::Keyword(KeywordAbility::Miracle),
            // CR 702.94a: The miracle alternative cost ({W}).
            AbilityDefinition::Miracle {
                cost: ManaCost { white: 1, ..Default::default() },
            },
            // The spell effect: destroy all creatures (approximates "put on bottom of
            // their owners' libraries" — owners' library routing deferred to M10+).
            AbilityDefinition::Spell {
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::AllCreatures,
                    cant_be_regenerated: false,
                },
                targets: vec![],
                modes: None,
                cant_be_countered: false,
            },
        ],
        ..Default::default()
    }
}
