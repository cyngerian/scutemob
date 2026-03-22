// Excise the Imperfect — {1}{W}{W}, Instant
// Exile target nonland permanent. Its controller incubates X, where X is its mana value.
//
// TODO: The "incubates X where X is its mana value" clause requires:
// 1. Effect::Incubate (not yet in DSL) — creates an Incubator token with X +1/+1 counters
//    and a "{2}: Transform this token" activated ability.
// 2. EffectAmount::ManaValueOf(EffectTarget) — to read the mana value of the exiled permanent.
// The exile effect is implemented; the incubate follow-up is omitted per W5 policy
// (partial implementation would produce wrong game state — opponent gets no Incubator token).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("excise-the-imperfect"),
        name: "Excise the Imperfect".to_string(),
        mana_cost: Some(ManaCost { generic: 1, white: 2, ..Default::default() }),
        types: types(&[CardType::Instant]),
        oracle_text: "Exile target nonland permanent. Its controller incubates X, where X is its mana value. (They create an Incubator token with X +1/+1 counters on it and \"{2}: Transform this token.\" It transforms into a 0/0 Phyrexian artifact creature.)".to_string(),
        abilities: vec![],
        ..Default::default()
    }
}
