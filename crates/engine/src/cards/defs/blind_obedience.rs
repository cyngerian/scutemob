// Blind Obedience — {1}{W}, Enchantment
// Extort
// Artifacts and creatures your opponents control enter tapped.
//
// TODO: "Artifacts and creatures your opponents control enter tapped" — needs a global
//   replacement effect for opponents' permanents entering tapped. ObjectFilter lacks
//   compound "opponent + artifact/creature" filter. Extort keyword is implemented.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("blind-obedience"),
        name: "Blind Obedience".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 1, ..Default::default() }),
        types: types(&[CardType::Enchantment]),
        oracle_text: "Extort (Whenever you cast a spell, you may pay {W/B}. If you do, each opponent loses 1 life and you gain that much life.)\nArtifacts and creatures your opponents control enter tapped.".to_string(),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Extort),
        ],
        ..Default::default()
    }
}
