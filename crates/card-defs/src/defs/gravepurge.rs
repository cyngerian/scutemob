// Gravepurge — {2}{B}, Instant
// Put any number of target creature cards from your graveyard on top of your library.
// Draw a card.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("gravepurge"),
        name: "Gravepurge".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Put any number of target creature cards from your graveyard on top of your \
                      library.\nDraw a card."
            .to_string(),
        abilities: vec![AbilityDefinition::Spell {
            // TODO: Multi-target graveyard-to-library not expressible. Draw only.
            effect: Effect::DrawCards {
                player: PlayerTarget::Controller,
                count: EffectAmount::Fixed(1),
            },
            targets: vec![],
            modes: None,
            cant_be_countered: false,
        }],
        completeness: Completeness::partial(
            "Blocked: (a) 'any number of target' is unbounded — TargetRequirement::UpToN takes a \
             fixed count, so only a bounded approximation exists; (b) no effect routes DECLARED \
             graveyard targets to library top — Effect::PutOnLibrary takes a count + source zone, \
             not declared targets. Graveyard targeting itself is NOT blocked \
             (TargetRequirement::TargetCardInYourGraveyard, PB-10). Def currently implements only \
             'Draw a card' and declares no targets.",
        ),
        ..Default::default()
    }
}
