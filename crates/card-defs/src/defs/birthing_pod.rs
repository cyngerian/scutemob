// Birthing Pod — {3}{G/P} Artifact
// {1}{G/P}, {T}, Sacrifice a creature: Search your library for a creature card with mana value equal to
//   1 plus the sacrificed creature's mana value, put that card onto the battlefield, then shuffle.
//   Activate only as a sorcery.
//
// PB-OS8: TargetFilter.min_cmc_amount (runtime LOWER-BOUND cap, mirror of the existing
// max_cmc_amount) now ships, so the "mana value EQUAL TO 1 + the sacrificed creature's MV"
// filter is expressible (max_cmc_amount == min_cmc_amount == Sum(Fixed(1),
// ManaValueOfSacrificedCreature); reference shape: eldritch_evolution.rs, birthing_ritual.rs).
//
// PB-RS2 (closes OOS-OS8-1): the second blocker -- Phyrexian mana in an ACTIVATED ability's
// cost was unsupported in rules/abilities.rs's payment path (it paid the RAW ManaCost, so a
// {G/P} pip's mana_value() > 0 passed the payment gate but can_spend/spend never read
// cost.phyrexian, charging it for free) -- is now closed. handle_activate_ability flattens
// hybrid/Phyrexian choices via ManaCost::flatten_hybrid_phyrexian before paying, exactly as
// rules/casting.rs already did for spells; a `false` in phyrexian_life_payments pays {G/P}
// with green mana, `true` pays it with 2 life (CR 107.4f, CR 119.4). Both blockers are now
// closed; this card flips to Complete.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("birthing-pod"),
        name: "Birthing Pod".to_string(),
        mana_cost: Some(ManaCost {
            generic: 3,
            phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "({G/P} can be paid with either {G} or 2 life.)\n{1}{G/P}, {T}, Sacrifice a \
                      creature: Search your library for a creature card with mana value equal to \
                      1 plus the sacrificed creature's mana value, put that card onto the \
                      battlefield, then shuffle. Activate only as a sorcery."
            .to_string(),
        abilities: vec![AbilityDefinition::Activated {
            // {1}{G/P}, {T}, Sacrifice a creature.
            cost: Cost::Sequence(vec![
                Cost::Mana(ManaCost {
                    generic: 1,
                    phyrexian: vec![PhyrexianMana::Single(ManaColor::Green)],
                    ..Default::default()
                }),
                Cost::Tap,
                Cost::Sacrifice(TargetFilter {
                    has_card_type: Some(CardType::Creature),
                    ..Default::default()
                }),
            ]),
            // Search for a creature card with mana value EQUAL TO 1 plus the sacrificed
            // creature's mana value (PB-OS8: paired max_cmc_amount/min_cmc_amount), put it
            // onto the battlefield, then shuffle (CR 601.2f-style explicit Shuffle, mirroring
            // eldritch_evolution.rs -- SearchLibrary's own shuffle_before_placing only
            // shuffles BEFORE placing, not after).
            effect: Effect::Sequence(vec![
                Effect::SearchLibrary {
                    player: PlayerTarget::Controller,
                    filter: TargetFilter {
                        has_card_type: Some(CardType::Creature),
                        max_cmc_amount: Some(Box::new(EffectAmount::Sum(
                            Box::new(EffectAmount::Fixed(1)),
                            Box::new(EffectAmount::ManaValueOfSacrificedCreature),
                        ))),
                        min_cmc_amount: Some(Box::new(EffectAmount::Sum(
                            Box::new(EffectAmount::Fixed(1)),
                            Box::new(EffectAmount::ManaValueOfSacrificedCreature),
                        ))),
                        ..Default::default()
                    },
                    reveal: false,
                    destination: ZoneTarget::Battlefield { tapped: false },
                    shuffle_before_placing: false,
                    also_search_graveyard: false,
                },
                Effect::Shuffle {
                    player: PlayerTarget::Controller,
                },
            ]),
            // "Activate only as a sorcery."
            timing_restriction: Some(TimingRestriction::SorcerySpeed),
            targets: vec![],
            activation_condition: None,
            activation_zone: None,
            once_per_turn: false,
            modes: None,
        }],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
