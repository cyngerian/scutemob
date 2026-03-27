// The Eternal Wanderer — {4}{W}{W}, Legendary Planeswalker (no subtype)
// Static: no more than one creature can attack it each combat.
// +1: Exile up to one target artifact or creature, return at beginning of owner's next end step.
// 0: Create a 2/2 white Samurai token with double strike.
// −4: For each player, choose a creature they control; each player sacrifices the rest.
//
// TODO: Static "no more than one creature can attack The Eternal Wanderer" —
//   attack restriction keyed on a specific planeswalker not in DSL.
// TODO: −4 mass sacrifice per-player choice — choose one creature per player then
//   sacrifice the rest; no DSL support for per-player selection and conditional sacrifice.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("the-eternal-wanderer"),
        name: "The Eternal Wanderer".to_string(),
        mana_cost: Some(ManaCost { generic: 4, white: 2, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Planeswalker], &[]),
        oracle_text: "No more than one creature can attack The Eternal Wanderer each combat.\n+1: Exile up to one target artifact or creature. Return that card to the battlefield under its owner's control at the beginning of that player's next end step.\n0: Create a 2/2 white Samurai creature token with double strike.\n\u{2212}4: For each player, choose a creature that player controls. Each player sacrifices all creatures they control not chosen this way.".to_string(),
        starting_loyalty: Some(5),
        abilities: vec![
            // TODO: static attack restriction (no more than one creature can attack this PW)
            // +1: Exile up to one target artifact or creature. Return it at beginning of
            // that player's next end step.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::ExileWithDelayedReturn {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    return_timing: crate::state::stubs::DelayedTriggerTiming::AtOwnersNextEndStep,
                    return_tapped: false,
                    return_to: crate::cards::card_definition::DelayedReturnDestination::Battlefield,
                },
                targets: vec![TargetRequirement::TargetPermanentWithFilter(TargetFilter {
                    has_card_types: vec![CardType::Artifact, CardType::Creature],
                    ..Default::default()
                })],
            },
            // 0: create 2/2 Samurai with double strike
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Zero,
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Samurai".to_string(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Samurai".to_string())].into_iter().collect(),
                        colors: [Color::White].into_iter().collect(),
                        power: 2,
                        toughness: 2,
                        count: 1,
                        supertypes: im::OrdSet::new(),
                        keywords: [KeywordAbility::DoubleStrike].into_iter().collect(),
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                        ..Default::default()
                    },
                },
                targets: vec![],
            },
            // −4: per-player choose + sacrifice rest (not expressible)
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(4),
                effect: Effect::Nothing,
                targets: vec![],
            },
        ],
        ..Default::default()
    }
}
