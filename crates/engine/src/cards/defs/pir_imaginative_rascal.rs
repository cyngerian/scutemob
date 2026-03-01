// Pir, Imaginative Rascal — {2}{G}, Legendary Creature — Human 1/1
// Partner with Toothy, Imaginary Friend (ETB trigger handled by PartnerWith keyword).
// Counter-doubling replacement effect: "If one or more counters would be put on a permanent
// your team controls, that many plus one of each of those kinds of counters are put on that
// permanent instead." — CR 614.1 replacement effect.
// TODO: Counter-doubling is not yet representable in the DSL. Requires a new
// ReplacementTrigger::WouldPutCountersOnPermanent { team_filter: bool } and
// ReplacementModification::AddOneToEachCounterKind variant in replacement_effect.rs.
// Until those variants exist, only the PartnerWith keyword is encoded here.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("pir-imaginative-rascal"),
        name: "Pir, Imaginative Rascal".to_string(),
        mana_cost: Some(ManaCost { generic: 2, green: 1, ..Default::default() }),
        types: full_types(&[SuperType::Legendary], &[CardType::Creature], &["Human"]),
        oracle_text:
            "Partner with Toothy, Imaginary Friend (When this creature enters the battlefield, \
             target player may search their library for a card named Toothy, Imaginary Friend, \
             reveal it, put it into their hand, then shuffle.)\n\
             If one or more counters would be put on a permanent your team controls, that many \
             plus one of each of those kinds of counters are put on that permanent instead."
                .to_string(),
        power: Some(1),
        toughness: Some(1),
        abilities: vec![
            // CR 702.124j: "Partner with [name]" — ETB trigger searches for named partner.
            AbilityDefinition::Keyword(KeywordAbility::PartnerWith(
                "Toothy, Imaginary Friend".to_string(),
            )),
            // TODO: Counter-doubling replacement effect — requires
            // ReplacementTrigger::WouldPutCountersOnPermanent and
            // ReplacementModification::AddOneToEachCounterKind (not yet in DSL).
        ],
    }
}
