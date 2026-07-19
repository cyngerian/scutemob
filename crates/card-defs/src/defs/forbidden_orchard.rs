// Forbidden Orchard — Land
// {T}: Add one mana of any color.
// Whenever you tap this land for mana, target opponent creates a 1/1 colorless Spirit creature token.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("forbidden-orchard"),
        name: "Forbidden Orchard".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}: Add one mana of any color.\nWhenever you tap this land for mana, \
                      target opponent creates a 1/1 colorless Spirit creature token."
            .to_string(),
        abilities: vec![
            // {T}: Add one mana of any color.
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor {
                    player: PlayerTarget::Controller,
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
                once_per_turn: false,
                modes: None,
            },
            // CR 605.5a / CR 605.1b: "Whenever you tap this land for mana, target opponent
            // creates a 1/1 colorless Spirit creature token."
            // This trigger has a target, so it is NOT a mana ability (CR 605.5a) — it goes
            // on the stack normally. The trigger fires from the mana ability activation.
            //
            // OOS-EF6-1 (PB-OS3, closed): `fire_mana_triggered_abilities` (rules/mana.rs)
            // now queues this trigger as `PendingTriggerKind::CardDefETB` (was `Normal`),
            // whose target/effect resolution both read `def.abilities.get(ability_index)` —
            // the raw def index this trigger is dispatched with — so the declared
            // `TargetRequirement::TargetOpponent` below now resolves correctly and
            // `TokenSpec.recipient` routes the Spirit to that opponent. Both halves of the
            // card now compose: PB-EF12 fixed the mana ability's real-colour choice; PB-OS3
            // fixed this trigger's target dispatch.
            AbilityDefinition::Triggered {
                once_per_turn: false,
                trigger_condition: TriggerCondition::WhenTappedForMana {
                    source_filter: ManaSourceFilter::This,
                },
                effect: Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Spirit".to_string(),
                        power: 1,
                        toughness: 1,
                        colors: OrdSet::new(),
                        card_types: [CardType::Creature].iter().copied().collect(),
                        subtypes: [SubType("Spirit".into())].iter().cloned().collect(),
                        count: EffectAmount::Fixed(1),
                        recipient: PlayerTarget::DeclaredTarget { index: 0 },
                        ..Default::default()
                    },
                },
                intervening_if: None,
                targets: vec![TargetRequirement::TargetOpponent],
                modes: None,
                trigger_zone: None,
            },
        ],
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
