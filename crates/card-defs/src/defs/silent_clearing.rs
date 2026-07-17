// Silent Clearing — Land
// {T}, Pay 1 life: Add {W} or {B}.
// {1}, {T}, Sacrifice this land: Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("silent-clearing"),
        name: "Silent Clearing".to_string(),
        mana_cost: None,
        types: types(&[CardType::Land]),
        oracle_text: "{T}, Pay 1 life: Add {W} or {B}.\n{1}, {T}, Sacrifice this land: Draw a card.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![Cost::Tap, Cost::PayLife(1)]),
                effect: Effect::AddManaChoice {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            AbilityDefinition::Activated {
                cost: Cost::Sequence(vec![
                    Cost::Mana(ManaCost { generic: 1, ..Default::default() }),
                    Cost::Tap,
                    Cost::SacrificeSelf,
                ]),
                effect: Effect::DrawCards {
                    player: PlayerTarget::Controller,
                    count: EffectAmount::Fixed(1),
                },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
        ],
        completeness: Completeness::known_wrong(
            "SR-33: adds {C}, not {W} or {B}. `Effect::AddManaChoice` has no field for which \
             colours are legal and its only execution site adds one colorless mana \
             (effects/mod.rs, the arm it shares with AddManaAnyColor), so \
             '{T}, Pay 1 life: Add {W} or {B}' produces a colour this land does not print. Blocked twice \
             over: the cost is {T} + Pay 1 life, and enrich_spec_from_def only lowers \
             `Cost::Tap` into a ManaAbility, so it is not a mana ability either (CR \
             605.1a). Needs a colour list on the variant (or per-colour abilities) AND \
             an activation cost on ManaAbility.",
        ),
        ..Default::default()
    }
}
