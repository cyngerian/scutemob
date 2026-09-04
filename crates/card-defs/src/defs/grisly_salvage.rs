// Grisly Salvage — {B}{G}, Instant
// Reveal the top five cards of your library. You may put a creature or land card
// from among them into your hand. Put the rest into your graveyard.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("grisly-salvage"),
        name: "Grisly Salvage".to_string(),
        mana_cost: Some(ManaCost {
            black: 1,
            green: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Reveal the top five cards of your library. You may put a creature or land \
                      card from among them into your hand. Put the rest into your graveyard."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // `Effect::RevealAndRoute` routes ALL matching cards (mandatory) — printed text is
            // "You may put A creature or land card" (at most one, optional). Use the put-≤1
            // sibling instead (see its doc in card-types/src/cards/card_definition.rs).
            // Note: neither primitive emits a distinct "reveal" event (only zone-move events),
            // so modelling this as "look at" rather than "reveal" loses no simulated behaviour.
            // NOTE (PB-DX4 fix cycle, review Finding 5): `optional: true` recorded the
            // printed "you **may**" structurally while it was **INERT** (the
            // `LookAtTopThenPlace` arm in `effects/mod.rs` destructured `optional: _`,
            // pre-existing **OOS-DP10-5**) -- filed as the class **OOS-DX4-5** rather than
            // settled per-card, because four other `Complete` defs shipped in exactly this
            // position (`birthing_ritual`, `growing_rites_of_itlimoc`, `risen_reef` and this
            // one's sibling). **CLOSED by PB-DX35** (`scutemob-227`, 2026-09-04):
            // `optional` is real now — a nonempty candidate set asks
            // `EffectChoiceQuestion::ChooseObject { count: 1, up_to: true, .. }` on the
            // CR 608.2d suspend-and-replay channel, and a decline routes the card to
            // `rest_to` (here, the graveyard) same as a no-match top-N always did. What
            // PB-DX4 fixed here is the arity: the previous `RevealAndRoute` routed EVERY
            // match, so the printed "a card" was wrong too.
            effect: Effect::LookAtTopThenPlace {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(5),
                filter: TargetFilter {
                    has_card_types: vec![CardType::Creature, CardType::Land],
                    ..Default::default()
                },
                place_cost: None,
                destination: ZoneTarget::Hand {
                    owner: PlayerTarget::Controller,
                },
                rest_to: ZoneTarget::Graveyard {
                    owner: PlayerTarget::Controller,
                },
                optional: true,
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        ..Default::default()
    }
}
