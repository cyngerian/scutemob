// Ashnod's Altar — {3}, Artifact.
// "Sacrifice a creature: Add {C}{C}."
// CR 602.2: Activated mana ability with sacrifice cost.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("ashnods-altar"),
        name: "Ashnod's Altar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Sacrifice a creature: Add {C}{C}.".to_string(),
        abilities: vec![AbilityDefinition::Activated {
            cost: Cost::Sacrifice(TargetFilter {
                has_card_type: Some(CardType::Creature),
                ..Default::default()
            }),
            effect: Effect::AddMana {
                player: PlayerTarget::Controller,
                mana: mana_pool(0, 0, 0, 0, 0, 2),
            },
            timing_restriction: None,
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
        }],
        completeness: Completeness::partial(
            "CR 605.1a/605.3b: 'Sacrifice a creature: Add {C}{C}' is a mana ability (no target, \
             adds mana, not loyalty) but is registered as a stack-using activated ability. The \
             MANA is correct (probed: +2 colorless). Blocked because Cost::Sacrifice(filter) \
             sacrifices ANOTHER permanent and needs a caller-supplied ObjectId, which \
             Command::TapForMana { player, source, ability_index } has no payload for — the \
             Krark-Clan Ironworks class (plan sr34 §2/§8 item 5). Fixing it means widening the \
             Command, not the lowering gate.",
        ),
        ..Default::default()
    }
}
