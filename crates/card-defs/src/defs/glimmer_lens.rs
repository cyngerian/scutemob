// Glimmer Lens — {1}{W}, Artifact — Equipment
// For Mirrodin! (When this Equipment enters, create a 2/2 red Rebel creature token,
// then attach this to it.)
// Whenever equipped creature and at least one other creature attack, draw a card.
// Equip {1}{W}
//
// RE-VERIFIED 2026-08-11 (PB-DX26 fix cycle, review Finding 4): "For Mirrodin!" IS
// expressible — `Effect::CreateTokenAndAttachSource` on a `WhenEntersBattlefield` trigger
// (`card_definition.rs`, executed in `effects/mod.rs`). The old TODO saying otherwise was
// stale, and this def's own `completeness` note has said so since before this batch.
// TODO: still genuinely blocked — "Whenever equipped creature and at least one other
//   creature attack" has no `TriggerCondition` (re-checked 2026-08-11).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("glimmer-lens"),
        name: "Glimmer Lens".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(&[], &[CardType::Artifact], &["Equipment"]),
        oracle_text: "For Mirrodin! (When this Equipment enters, create a 2/2 red Rebel creature \
                      token, then attach this to it.)\nWhenever equipped creature and at least \
                      one other creature attack, draw a card.\nEquip {1}{W}"
            .to_string(),
        abilities: vec![
            // TODO: the attack trigger only — "For Mirrodin!" is expressible and unauthored
            // (see the header note, re-verified 2026-08-11).
            AbilityDefinition::Keyword(KeywordAbility::Equip),
            // Equip {1}{W}: attach this Equipment to target creature you control.
            // CR 702.6b: Equip is an activated ability; CR 702.6d: sorcery speed only.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 1,
                    white: 1,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // PB-DX26 (OOS-CARDS1-3) / CR 702.6a: "Equip {1}{W}" means "[Cost]: Attach this
                // permanent to target creature you control." Printed line MCP-verified as
                // plain "Equip {1}{W}" with no CR 702.6c quality restriction, so the
                // requirement is the unmodified 702.6a one.
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::You,
                    ..Default::default()
                })],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
        ],
        completeness: Completeness::partial(
            "Blocked on the attack trigger only: 'Whenever equipped creature and at least one \
             other creature attack' has no TriggerCondition \
             (WhenEquippedCreatureDealsCombatDamageToPlayer is a damage trigger; \
             WheneverYouAttack is unfiltered). 'For Mirrodin!' is NOT blocked — \
             Effect::CreateTokenAndAttachSource on a WhenEntersBattlefield trigger expresses it. \
             Equip {1}{W} is now authored as an Activated/AttachEquipment ability.",
        ),
        ..Default::default()
    }
}
