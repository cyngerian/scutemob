// Birthing Ritual — {1}{G}, Enchantment
// "At the beginning of your end step, if you control a creature, look at the top
//  seven cards of your library. Then you may sacrifice a creature. If you do, you
//  may put a creature card with mana value X or less from among those cards onto
//  the battlefield, where X is 1 plus the sacrificed creature's mana value. Put
//  the rest on the bottom of your library in a random order."
//
// PB-OS8 (closes OOS-EF10-1): the "look at the top seven, put at most one matching
// creature from that subset onto the battlefield, rest to bottom" dig is now
// expressible via Effect::LookAtTopThenPlace. `place_cost: Cost::Sacrifice(creature)`
// is the interposed "you may sacrifice a creature" (CR 118.12) — paid AFTER the
// look, BEFORE placing, and its LKI parameterizes `filter.max_cmc_amount = 1 +
// ManaValueOfSacrificedCreature` (CR 202.3/608.2h). Deterministic "pay when able"
// (architecture invariant #9): the sacrifice fires whenever a creature is
// available, even into a whiff, same as every other MayPayThenEffect-shaped
// Complete card. The intervening-if re-check at resolution (CR 603.4) and the
// {X}=0 mana-value rule for X-cost cards among the seven are handled by existing
// infrastructure (TriggerCondition/ManaValueOfSacrificedCreature). Rest-to-bottom
// "in a random order" is realized as ObjectId-ascending deterministic placement,
// the M7 precedent already used by RevealAndRoute/Scry/PutOnLibrary (NO rand).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birthing-ritual"),
        name: "Birthing Ritual".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "At the beginning of your end step, if you control a creature, look at the \
                      top seven cards of your library. Then you may sacrifice a creature. If you \
                      do, you may put a creature card with mana value X or less from among those \
                      cards onto the battlefield, where X is 1 plus the sacrificed creature's \
                      mana value. Put the rest on the bottom of your library in a random order."
            .to_string(),
        abilities: vec![
            // CR 603.3/603.4: "At the beginning of your end step, if you control a
            // creature" — end-step trigger with intervening-if.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::AtBeginningOfYourEndStep,
                intervening_if: Some(Condition::YouControlNOrMoreWithFilter {
                    count: 1,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    },
                }),
                effect: Effect::LookAtTopThenPlace {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(7),
                    place_cost: Some(Box::new(Cost::Sacrifice(TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        ..Default::default()
                    }))),
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                            Box::new(EffectAmount::Fixed(1)),
                            Box::new(EffectAmount::ManaValueOfSacrificedCreature),
                        ))),
                        ..Default::default()
                    },
                    destination: ZoneTarget::Battlefield { tapped: false },
                    rest_to: ZoneTarget::Library {
                        owner: PlayerTarget::Controller,
                        position: LibraryPosition::Bottom,
                    },
                    optional: true,
                },
                targets: vec![],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
