// Necrogen Rotpriest — {2}{B}{G}, Creature — Phyrexian Zombie Cleric 1/5
// Toxic 2
// Whenever a creature you control with toxic deals combat damage to a player, that player gets an additional poison counter.
// {1}{B}{G}: Target creature you control with toxic gains deathtouch until end of turn.
// TODO: DSL gap — "whenever a creature you control with toxic deals combat damage to a player"
// requires TriggerCondition::WheneverCreatureYouControlWithKeywordDealsCombatDamage, which
// doesn't exist.
// TODO: DSL gap — activated ability targeting a creature you control with toxic and granting
// deathtouch requires TargetFilter::CreatureYouControlWithKeyword, which doesn't exist.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("necrogen-rotpriest"),
        name: "Necrogen Rotpriest".to_string(),
        mana_cost: Some(ManaCost { generic: 2, black: 1, green: 1, ..Default::default() }),
        types: creature_types(&["Phyrexian", "Zombie", "Cleric"]),
        oracle_text: "Toxic 2 (Players dealt combat damage by this creature also get two poison counters.)\nWhenever a creature you control with toxic deals combat damage to a player, that player gets an additional poison counter.\n{1}{B}{G}: Target creature you control with toxic gains deathtouch until end of turn.".to_string(),
        power: Some(1),
        toughness: Some(5),
        abilities: vec![
            AbilityDefinition::Keyword(KeywordAbility::Toxic(2)),
            // TODO: triggered — whenever a creature you control with toxic deals combat damage
            // to a player, that player gets an additional poison counter.
            // DSL gap: no TriggerCondition filtering on creature's keywords.
            // TODO: activated — {1}{B}{G}: target creature you control with toxic gains deathtouch until end of turn.
            // DSL gap: no TargetFilter::CreatureYouControlWithKeyword(Toxic).
        ],
        completeness: Completeness::partial("The named types are NOT the blocker: TriggerCondition::WheneverCreatureYouControlDealsCombatDamageToPlayer takes an Option<TargetFilter> (card_definition.rs:3232) and TargetFilter::has_keywords exists (card_definition.rs:2813). The live blocker is parameterized-keyword matching: has_keywords is matched by exact set membership (effects/mod.rs:7957, chars.keywords.contains(kw)), and Toxic is KeywordAbility::Toxic(u32) — so has_keywords={Toxic(2)} misses Toxic(1)/Toxic(3) and 'a creature you control with toxic' (any N) cannot be expressed. Blocks both the trigger and the {1}{B}{G} deathtouch grant."),
        ..Default::default()
    }
}
