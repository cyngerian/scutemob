// 79. Qarsi Sadist — {1}{B}, Creature — Human Cleric 1/3; Exploit.
// CR 702.110a: When this enters, you may sacrifice a creature.
// CR 702.110b: When this creature exploits a creature, target opponent loses 2 life
// and you gain 2 life.  <-- NOT AUTHORED; see the completeness note.
//
// PB-DX27 (2026-08-13), OOS-CARDS2-10: this def was `Completeness::Complete` and
// deck-legal while silently omitting the whole second printed clause, from both
// `oracle_text` and `abilities`. It is DEMOTED rather than repaired, because the
// clause is genuinely inexpressible — see the note. Its two sibling Exploit defs
// (`fell_stinger`, `sidisi_undead_vizier`) were already `partial` for exactly this
// reason; this def was the outlier that nobody had ruled on.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("qarsi-sadist"),
        name: "Qarsi Sadist".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            black: 1,
            ..Default::default()
        }),
        types: types_sub(&[CardType::Creature], &["Human", "Cleric"]),
        oracle_text: "Exploit (When this creature enters, you may sacrifice a creature.)\nWhen \
                      this creature exploits a creature, target opponent loses 2 life and you \
                      gain 2 life."
            .to_string(),
        power: Some(1),
        toughness: Some(3),
        abilities: vec![AbilityDefinition::Keyword(KeywordAbility::Exploit)],
        color_indicator: None,
        back_face: None,
        spell_cost_modifiers: vec![],
        self_cost_reduction: None,
        starting_loyalty: None,
        adventure_face: None,
        meld_pair: None,
        spell_additional_costs: vec![],
        activated_ability_cost_reductions: vec![],
        cant_be_countered: false,
        self_exile_on_resolution: false,
        self_shuffle_on_resolution: false,
        completeness: Completeness::partial(
            "Blocked on the secondary exploit trigger (CR 702.110b): \
             TriggerCondition::WhenThisExploitsACreature does not exist in the DSL — grep \
             card_definition.rs for `Exploit` returns ZERO hits, so there is no trigger to \
             hang 'target opponent loses 2 life and you gain 2 life' on. Both halves of the \
             payoff ARE expressible (Effect::LoseLife / Effect::GainLife with \
             TargetRequirement::TargetOpponent); the trigger is the whole gap. Secondarily, \
             Exploit's own ETB trigger unconditionally declines the sacrifice at \
             resolution.rs:4095-4104 pending an interactive Command::ExploitCreature, so \
             nothing is ever exploited and the trigger could not fire even if it existed. \
             PB-DX27 (2026-08-13) demoted this from Complete: it is the same blocker \
             fell_stinger and sidisi_undead_vizier already carry, and this def was the only \
             one of the corpus's three Exploit cards claiming Complete.",
        ),
    }
}
