// Patriar's Seal — {3}, Artifact
// {T}: Add one mana of any color.
// {1}, {T}: Untap target legendary creature you control.
//
// TODO: "{1}, {T}: Untap target legendary creature" — TargetFilter lacks has_supertype
//   field for legendary filtering. Implementing only the mana ability.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("patriars-seal"),
        name: "Patriar's Seal".to_string(),
        mana_cost: Some(ManaCost { generic: 3, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{T}: Add one mana of any color.\n{1}, {T}: Untap target legendary creature you control.".to_string(),
        abilities: vec![
            AbilityDefinition::Activated {
                cost: Cost::Tap,
                effect: Effect::AddManaAnyColor { player: PlayerTarget::Controller },
                timing_restriction: None,
                targets: vec![],
                activation_condition: None,
                activation_zone: None,
            once_per_turn: false,
            },
            // TODO: {1}, {T}: Untap target legendary creature (TargetFilter lacks has_supertype)
        ],
        completeness: Completeness::partial("Second ability unimplemented. Blocker is stale: TargetFilter.legendary exists (card_definition.rs:2858) and is enforced in matches_filter (effects/mod.rs:8045); see eiganjo_seat_of_the_empire.rs / boseiju_who_endures.rs for shipped usage. Author '{1}, {T}: Untap target legendary creature you control' as Cost::Sequence([Mana{generic:1}, Tap]) + Effect::UntapPermanent on TargetCreatureWithFilter(TargetFilter{legendary: true, controller: You})."),
        ..Default::default()
    }
}
