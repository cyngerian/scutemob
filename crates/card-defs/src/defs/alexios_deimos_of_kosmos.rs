// Alexios, Deimos of Kosmos — {3}{R}, Legendary Creature — Human Berserker 4/4
// Trample
// Alexios attacks each combat if able, can't be sacrificed, and can't attack its owner.
// At the beginning of each player's upkeep, that player gains control of Alexios, untaps it,
// and puts a +1/+1 counter on it. It gains haste until end of turn.
//
// Trample and "attacks each combat if able" are implemented.
// PB-AC8 built GameRestriction::CantBeSacrificed and GameRestriction::CantAttackOwner
// (AbilityDefinition::StaticRestriction) — both restrictions now exist in the DSL and
// are no longer engine-blocked, but they have not yet been wired onto this card def
// (deferred backfill; card remains PARTIAL below for that reason plus the genuine gap).
// Remaining genuine DSL gap:
// - Upkeep trigger: GainControl + UntapPermanent + AddCounters + GainHaste until end of turn
//   targeting EACH player's upkeep (not just the controller's) — no ForEachPlayer/ambient
//   upkeep trigger scope exists in the DSL today.
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
            // CR 508.1d: Attacks each combat if able.
            AbilityDefinition::Keyword(KeywordAbility::MustAttackEachCombat),
            // TODO: "can't be sacrificed" / "can't attack its owner" — primitives now exist
            // (GameRestriction::CantBeSacrificed / CantAttackOwner, PB-AC8) but are not yet
            // authored onto this card; deferred to a future backfill pass.
            // TODO: Upkeep trigger (each player's upkeep): GainControl + untap + AddCounters + haste
            // — genuine DSL gap, no ForEachPlayer/ambient upkeep trigger scope exists.
        ],
        completeness: Completeness::partial("'can't be sacrificed' / 'can't attack its owner' are unwired but unblocked — add two AbilityDefinition::StaticRestriction entries (GameRestriction::CantBeSacrificed / CantAttackOwner, PB-AC8). Card remains partial for the each-player's-upkeep clause: 'that player gains control' has no PlayerTarget for the upkeep player."),
        ..Default::default()
    }
}
