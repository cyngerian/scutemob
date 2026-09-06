// Nature's Claim — {G} Instant; destroy target artifact or enchantment. Its controller gains 4 life.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("natures-claim"),
        name: "Nature's Claim".to_string(),
        mana_cost: Some(ManaCost {
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Destroy target artifact or enchantment. Its controller gains 4 life."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // Destroy the artifact or enchantment, then its controller gains 4 life.
            // ControllerOf(DeclaredTarget{0}) names the destroyed permanent's controller,
            // not the spell's. NOTE: this was ASPIRATIONAL until `scutemob-255` -- the
            // destroy retires the target's ObjectId (CR 400.7) before this second clause
            // runs, so ControllerOf resolved to an EMPTY player list and NOBODY gained the
            // 4 life, on a def marked Complete. The CR 608.2h fallback in
            // `resolve_player_target_list` is what makes the line true. Pinned by
            // `primitives::ll1_token_recipient_controller_of::t4`.
            effect: Effect::Sequence(vec![
                Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                Effect::GainLife {
                    player: PlayerTarget::ControllerOf(Box::new(EffectTarget::DeclaredTarget {
                        index: 0,
                    })),
                    amount: EffectAmount::Fixed(4),
                },
            ]),
            targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                has_card_types: vec![CardType::Artifact, CardType::Enchantment],
                ..Default::default()
            })],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
