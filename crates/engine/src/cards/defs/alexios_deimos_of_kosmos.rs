// Alexios, Deimos of Kosmos — {3}{R}, Legendary Creature — Human Berserker 4/4
// Trample
// Alexios attacks each combat if able, can't be sacrificed, and can't attack its owner.
// At the beginning of each player's upkeep, that player gains control of Alexios, untaps it,
// and puts a +1/+1 counter on it. It gains haste until end of turn.
//
// Trample is implemented. Other abilities require DSL gaps:
// - "attacks each combat if able" — no forced-attack static in DSL
// - "can't be sacrificed" — no restriction effect for sacrifice
// - "can't attack its owner" — no attack restriction by target controller
// - Upkeep trigger: GainControl + UntapPermanent + AddCounters + GainHaste until end of turn
//   targeting each player's upkeep (not just controller's) — no ForEachPlayer upkeep trigger
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("alexios-deimos-of-kosmos"),
        name: "Alexios, Deimos of Kosmos".to_string(),
        mana_cost: Some(ManaCost { generic: 3, red: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human", "Berserker"]),
        oracle_text: "Trample\nAlexios attacks each combat if able, can't be sacrificed, and can't attack its owner.\nAt the beginning of each player's upkeep, that player gains control of Alexios, untaps it, and puts a +1/+1 counter on it. It gains haste until end of turn.".to_string(),
        power: Some(4),
        toughness: Some(4),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Trample),
            // TODO: "attacks each combat if able" — forced-attack static not in DSL
            // TODO: "can't be sacrificed" — sacrifice restriction not in DSL
            // TODO: "can't attack its owner" — attack restriction by owner not in DSL
            // TODO: Upkeep trigger (each player's upkeep): GainControl + untap + AddCounters + haste
        ],
        ..Default::default()
    }
}
