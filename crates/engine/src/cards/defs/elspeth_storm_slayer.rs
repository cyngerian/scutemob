// Elspeth, Storm Slayer — {3}{W}{W} Legendary Planeswalker — Elspeth
// If one or more tokens would be created under your control, twice that many
// of those tokens are created instead.
// +1: Create a 1/1 white Soldier creature token.
// 0: Put a +1/+1 counter on each creature you control. Those creatures gain
//    flying until your next turn.
// −3: Destroy target creature an opponent controls with mana value 3 or greater.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("elspeth-storm-slayer"),
        name: "Elspeth, Storm Slayer".to_string(),
        mana_cost: Some(ManaCost { generic: 3, white: 2, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Planeswalker],
            &["Elspeth"],
        ),
        oracle_text: "If one or more tokens would be created under your control, twice that many of those tokens are created instead.\n+1: Create a 1/1 white Soldier creature token.\n0: Put a +1/+1 counter on each creature you control. Those creatures gain flying until your next turn.\n\u{2212}3: Destroy target creature an opponent controls with mana value 3 or greater.".to_string(),
        starting_loyalty: Some(5),
        abilities: vec![
            // Static: "If one or more tokens would be created under your control, twice that
            // many of those tokens are created instead." — CR 614.1 token-doubling replacement.
            // PlayerId(0) placeholder — bound to controller at registration.
            AbilityDefinition::Replacement {
                trigger: ReplacementTrigger::WouldCreateTokens {
                    controller_filter: PlayerFilter::Specific(PlayerId(0)),
                },
                modification: ReplacementModification::DoubleTokens,
                is_self: false,
                unless_condition: None,
            },
            // +1: Create a 1/1 white Soldier creature token.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(1),
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Soldier".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: [Color::White].into_iter().collect(),
                        supertypes: im::OrdSet::new(),
                        card_types: [CardType::Creature].into_iter().collect(),
                        subtypes: [SubType("Soldier".to_string())].into_iter().collect(),
                        keywords: im::OrdSet::new(),
                        count: 1,
                        tapped: false,
                        enters_attacking: false,
                        mana_color: None,
                        mana_abilities: vec![],
                        activated_abilities: vec![],
                    },
                },
                targets: vec![],
            },
            // 0: Put a +1/+1 counter on each creature you control.
            //    Those creatures gain flying until your next turn.
            // TODO: "gain flying until your next turn" duration not expressible
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Plus(0),
                effect: Effect::ForEach {
                    over: ForEachTarget::EachCreatureYouControl,
                    effect: Box::new(Effect::AddCounter {
                        target: EffectTarget::DeclaredTarget { index: 0 },
                        counter: CounterType::PlusOnePlusOne,
                        count: 1,
                    }),
                },
                targets: vec![],
            },
            // −3: Destroy target creature an opponent controls with MV 3 or greater.
            AbilityDefinition::LoyaltyAbility {
                cost: LoyaltyCost::Minus(3),
                effect: Effect::DestroyPermanent {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    cant_be_regenerated: false,
                },
                targets: vec![TargetRequirement::TargetCreatureWithFilter(TargetFilter {
                    controller: TargetController::Opponent,
                    min_cmc: Some(3),
                    ..Default::default()
                })],
            },
        ],
        ..Default::default()
    }
}
