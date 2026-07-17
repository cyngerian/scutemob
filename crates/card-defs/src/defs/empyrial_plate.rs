// Empyrial Plate — {2}, Artifact — Equipment
// Equipped creature gets +1/+1 for each card in your hand.
// Equip {2}
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("empyrial-plate"),
        name: "Empyrial Plate".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Artifact], &["Equipment"]),
        oracle_text: "Equipped creature gets +1/+1 for each card in your hand.\nEquip {2}"
            .to_string(),
        abilities: vec![
            // TODO: DSL gap — dynamic +1/+1 per card in hand. LayerModification::ModifyBoth
            // takes fixed i32, not EffectAmount. Needs dynamic LayerModification.
            AbilityDefinition::Keyword(KeywordAbility::Equip),
        ],
        completeness: Completeness::partial(
            "Rewire: AbilityDefinition::Static { ContinuousEffectDef { layer: PtModify, \
             modification: LayerModification::ModifyBothDynamic { amount: \
             Box::new(EffectAmount::HandSize { player: PlayerTarget::Controller }), negate: false \
             }, filter: EffectFilter::AttachedCreature, duration: WhileSourceOnBattlefield } }. \
             Verify first: layers.rs:1270-1275 resolves via the modified object's controller, not \
             the Equipment's — confirm 'your hand' resolves to the Equipment controller under \
             gain-control before marking Complete.",
        ),
        ..Default::default()
    }
}
