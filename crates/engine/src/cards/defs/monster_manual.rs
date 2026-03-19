// Monster Manual // Zoological Study — Adventure Artifact
//
// TODO: "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield" —
// activated ability requires targeting a card in the controller's hand. DSL gap:
// TargetRequirement has TargetCardInYourGraveyard and TargetCardInGraveyard but no
// TargetCardInHand variant. Once that is added, the effect would be:
//   Effect::MoveZone {
//       target: EffectTarget::DeclaredTarget { index: 0 },
//       to: ZoneTarget::Battlefield { tapped: false },
//       controller_override: None,
//   }
// The "you may" optional activation is standard (player can always decline to activate).
//
// TODO: Adventure half "Zoological Study" ({2}{G}, sorcery: Search your library for a creature
// card, put it onto the battlefield, then shuffle) — deferred per sub-batch 13m (Adventure
// framework requires AltCostKind::Adventure + split-card exile casting).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("monster-manual"),
        name: "Monster Manual // Zoological Study".to_string(),
        mana_cost: Some(ManaCost { generic: 3, green: 1, ..Default::default() }),
        types: types(&[CardType::Artifact]),
        oracle_text: "{1}{G}, {T}: You may put a creature card from your hand onto the battlefield.".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
