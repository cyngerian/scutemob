// Allied Strategies — {4}{U}, Sorcery
// Domain — Target player draws a card for each basic land type among lands they control.
//
// CR 305.6 / ability word "Domain": the effect magnitude equals the number of distinct basic
// land types (Plains, Island, Swamp, Mountain, Forest) among lands the TARGET PLAYER controls.
// "They" in the oracle text refers to the target player, not the caster.
// Uses layer-resolved characteristics via EffectAmount::DomainCount.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("allied-strategies"),
        name: "Allied Strategies".to_string(),
        mana_cost: Some(ManaCost { generic: 4, blue: 1, ..Default::default() }),
        types: types(&[CardType::Sorcery]),
        oracle_text: "Domain \u{2014} Target player draws a card for each basic land type among lands they control.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // CR 305.6: Domain — draw cards equal to the number of distinct basic land types
            // among lands the TARGET PLAYER controls (oracle: "lands they control").
            // DomainCount { player: DeclaredTarget { index: 0 } } counts the target player's lands.
            // DomainCount uses calculate_characteristics() so Layer 4 effects are accounted for.
            effect: Effect::DrawCards {
                player: PlayerTarget::DeclaredTarget { index: 0 },
                count: EffectAmount::DomainCount { player: PlayerTarget::DeclaredTarget { index: 0 } },
            },
            targets: vec![TargetRequirement::TargetPlayer],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
