// Kiki-Jiki, Mirror Breaker — {2}{R}{R}{R}, Legendary Creature — Goblin Shaman 2/2
// Haste
// {T}: Create a token that's a copy of target nonlegendary creature you control, except
// it has haste. Sacrifice it at the beginning of the next end step.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("kiki-jiki-mirror-breaker"),
        name: "Kiki-Jiki, Mirror Breaker".to_string(),
        mana_cost: Some(ManaCost { generic: 2, red: 3, ..Default::default() }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Goblin", "Shaman"],
        ),
        oracle_text: "Haste\n{T}: Create a token that's a copy of target nonlegendary creature you control, except it has haste. Sacrifice it at the beginning of the next end step.".to_string(),
        power: Some(2),
        toughness: Some(2),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Haste),
            // TODO: "Copy of target creature" token creation + delayed sacrifice not in DSL.
            //   Copy-token creation requires CreateCopyToken effect variant.
        ],
        ..Default::default()
    }
}
