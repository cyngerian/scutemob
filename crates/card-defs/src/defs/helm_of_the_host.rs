// Helm of the Host — {4}, Legendary Artifact — Equipment
// At the beginning of combat on your turn, create a token that's a copy of
// equipped creature, except the token isn't legendary. That token gains haste.
// Equip {5}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("helm-of-the-host"),
        name: "Helm of the Host".to_string(),
        mana_cost: Some(ManaCost {
            generic: 4,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Artifact],
            &["Equipment"],
        ),
        oracle_text: "At the beginning of combat on your turn, create a token that's a copy of \
                      equipped creature, except the token isn't legendary. That token gains \
                      haste.\nEquip {5}"
            .to_string(),
        abilities: vec![
            // At the beginning of combat on your turn, create a token that's a copy of
            // equipped creature, except the token isn't legendary. That token gains haste.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfCombat,
                effect: Effect::CreateTokenCopy {
                    source: EffectTarget::EquippedCreature,
                    enters_tapped_and_attacking: false,
                    except_not_legendary: true,
                    gains_haste: true,
                    delayed_action: None,
                },
                intervening_if: None,
                targets: vec![],

                modes: None,
                trigger_zone: None,
            },
            // Equip {5}: attach this Equipment to target creature you control.
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 5,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // CARDS-1 (OOS-M11-10) / CR 702.6a: "Equip {5}" means "[Cost]: Attach this
                // permanent to target creature you control." The printed line was MCP-verified
                // as plain "Equip {5}" -- no CR 702.6c quality restriction -- so the requirement is
                // the unmodified 702.6a one. This def is the ONE member of the 17-card equip
                // roster that already declared a requirement, so its repair is a TIGHTENING, not
                // an addition: it read a bare `TargetRequirement::TargetCreature`, which dropped
                // 702.6a's "you control" clause and so offered an opponent's creature as a legal
                // pick to every target-candidate query. The other 16 declared `targets: vec![]`.
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
        // PB-RS3: explicit marker. Previously had NO `completeness` field and was
        // `Complete` only by `#[default]` (card_definition.rs:196-200) -- that
        // implicitness is exactly why it shipped live-wrong (deck-legal, its only
        // real ability silently never fired -- `begin_combat` had no card-def scan
        // for TriggerCondition::AtBeginningOfCombat). Oracle-verified via MCP:
        // "At the beginning of combat on your turn, create a token that's a copy of
        // equipped creature, except the token isn't legendary. That token gains
        // haste." -- a faithful, unmodified translation. Making the marker explicit
        // converts a silent default into a reviewed assertion.
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
