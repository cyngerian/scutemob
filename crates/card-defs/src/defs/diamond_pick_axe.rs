// Diamond Pick-Axe — {R}, Artifact — Equipment
// Indestructible. Equipped creature gets +1/+1 and has "Whenever this creature attacks,
// create a Treasure token." Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("diamond-pick-axe"),
        name: "Diamond Pick-Axe".to_string(),
        mana_cost: Some(ManaCost {
            red: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Indestructible (Effects that say \"destroy\" don't destroy this \
                      Equipment.)\nEquipped creature gets +1/+1 and has \"Whenever this creature \
                      attacks, create a Treasure token.\" (It's an artifact with \"{T}, Sacrifice \
                      this token: Add one mana of any color.\")\nEquip {2}"
            .to_string(),
        abilities: vec![
            // Indestructible (self)
            AbilityDefinition::Keyword(KeywordAbility::Indestructible),
            // Static: equipped creature gets +1/+1
            AbilityDefinition::Static {
                continuous_effect: ContinuousEffectDef {
                    layer: EffectLayer::PtModify,
                    modification: LayerModification::ModifyBoth(1),
                    filter: EffectFilter::AttachedCreature,
                    duration: EffectDuration::WhileSourceOnBattlefield,
                    condition: None,
                },
            },
            // TODO: "Equipped creature has 'Whenever this creature attacks, create a Treasure token'"
            // — granting a triggered ability to the attached creature is not expressible in the DSL.
            // DSL gap: no GrantTrigger/GrantTriggeredAbility to attached creature.
            // Equip {2}
            AbilityDefinition::Activated {
                cost: Cost::Mana(ManaCost {
                    generic: 2,
                    ..Default::default()
                }),
                effect: Effect::AttachEquipment {
                    equipment: EffectTarget::Source,
                    target: EffectTarget::DeclaredTarget { index: 0 },
                },
                timing_restriction: Some(TimingRestriction::SorcerySpeed),
                // CARDS-1 (OOS-M11-10) / CR 702.6a: "Equip {2}" means "[Cost]: Attach this
                // permanent to target creature you control." The printed line was MCP-verified
                // as plain "Equip {2}" -- no CR 702.6c quality restriction -- so the requirement is
                // the unmodified 702.6a one. Authoring it is what makes the target announceable:
                // an empty `targets` list left `queries::ability_target_requirements` reporting
                // zero slots, so the browser picker never asked and the attach silently fizzled
                // with the cost paid.
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
            "'Equipped creature has 'Whenever this creature attacks, create a Treasure token'' — \
             granting a triggered ability to the...",
        ),
        ..Default::default()
    }
}
