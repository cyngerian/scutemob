// Chaos Warp — {2}{R} Instant; owner shuffles target permanent into their library,
// then reveals the top card; if it's a permanent card, put it onto the battlefield.
// TODO: DSL gap — "shuffle into library then reveal top; if permanent card, put onto
// battlefield" requires a reveal+conditional-ETB effect that does not exist. The
// shuffle-into-library portion uses MoveZone to the owner's library + Shuffle.
// The reveal-and-put-onto-battlefield portion is omitted (DSL gap).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("chaos-warp"),
        name: "Chaos Warp".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 1, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "The owner of target permanent shuffles it into their library, then reveals the top card of their library. If it's a permanent card, they put it onto the battlefield.".to_string(),
        abilities: vec![AbilityDefinition::Spell {
            effect: Effect::Sequence(vec![
                Effect::MoveZone {
                    target: EffectTarget::DeclaredTarget { index: 0 },
                    to: ZoneTarget::Library { owner: PlayerTarget::Controller, position: LibraryPosition::ShuffledIn },
                },
                // Shuffle is implicit with ShuffledIn position above
                // TODO: DSL gap — reveal top card and conditionally put it onto the
                // battlefield if it's a permanent card. No Effect variant exists for
                // "reveal top of library and put permanent cards onto battlefield".
            ]),
            targets: vec![TargetRequirement::TargetPermanent],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
