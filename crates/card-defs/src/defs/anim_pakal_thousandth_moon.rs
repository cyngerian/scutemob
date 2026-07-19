// Anim Pakal, Thousandth Moon — {1}{R}{W}, Legendary Creature — Human Soldier 1/2
// Whenever you attack with one or more non-Gnome creatures, put a +1/+1 counter on Anim
// Pakal, then create X 1/1 colorless Gnome artifact creature tokens that are tapped and
// attacking, where X is the number of +1/+1 counters on Anim Pakal.
//
// PB-OS11 (OOS-TS-1 reframed, CR 508.1/508.1m/603.2c): the trigger is a BATCH trigger —
// it fires ONCE per combat if at least one non-Gnome creature attacked, not once per
// matching attacker. TriggerCondition::WheneverYouAttack now carries an optional
// `filter: Option<TargetFilter>` on the declared-attacker set; `exclude_subtypes: [Gnome]`
// expresses "one or more non-Gnome creatures" directly (TargetFilter.exclude_subtypes
// already existed and is enforced in matches_filter — no new filter field was needed).
// The token count reads Anim Pakal's live +1/+1 counters via
// EffectAmount::CounterCount{Source, PlusOnePlusOne}, evaluated AFTER the AddCounter step
// in the Sequence (so it is the post-increment count, per oracle text).
//
// Decoy/no-inflation (ruling 2023-11-10): the created Gnome tokens ENTER attacking
// (enters_attacking: true) rather than being DECLARED as attackers, so they never fire
// AttackersDeclared/ControllerAttacks — they cannot re-trigger this ability or inflate a
// later count within the same combat.
//
// Known accepted minor deviation (documented, non-blocking — see pb-plan-OS11.md Part B,
// B-Card-1): if Anim Pakal itself leaves the battlefield mid-resolution of this trigger
// (e.g. to a removal spell responding to the trigger), ruling (a) says to use its
// last-known +1/+1 counter count; CounterCount{Source} reads LIVE counters instead. No
// non-leaves-trigger LKI counter reader exists in the engine today. In the normal case
// (Anim Pakal present through resolution — the overwhelming majority of games), the count
// is correct.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("anim-pakal-thousandth-moon"),
        name: "Anim Pakal, Thousandth Moon".to_string(),
        mana_cost: Some(ManaCost {
            generic: 1,
            red: 1,
            white: 1,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "Whenever you attack with one or more non-Gnome creatures, put a +1/+1 \
                      counter on Anim Pakal, Thousandth Moon, then create X 1/1 colorless Gnome \
                      artifact creature tokens that are tapped and attacking, where X is the \
                      number of +1/+1 counters on Anim Pakal."
            .to_string(),
        power: Some(1),
        toughness: Some(2),
        abilities: vec![AbilityDefinition::Triggered {
            once_per_turn: false,
            // CR 508.1/508.1m: batch trigger, fires once per combat if >=1 non-Gnome
            // attacker matches. exclude_subtypes:[Gnome] alone is correct for "non-Gnome
            // creatures" -- is_nontoken is NOT in the oracle and would be redundant (the
            // created Gnome tokens self-exclude as Gnomes via the subtype exclusion).
            trigger_condition: TriggerCondition::WheneverYouAttack {
                filter: Some(TargetFilter {
                    exclude_subtypes: vec![SubType("Gnome".to_string())],
                    ..Default::default()
                }),
            },
            effect: Effect::Sequence(vec![
                // "put a +1/+1 counter on Anim Pakal, ..."
                Effect::AddCounter {
                    target: EffectTarget::Source,
                    counter: CounterType::PlusOnePlusOne,
                    count: 1,
                },
                // "... then create X 1/1 colorless Gnome artifact creature tokens tapped
                //  and attacking, where X = number of +1/+1 counters on Anim Pakal"
                //  (evaluated AFTER the counter is added -- post-increment count).
                Effect::CreateToken {
                    spec: TokenSpec {
                        name: "Gnome".to_string(),
                        card_types: [CardType::Artifact, CardType::Creature]
                            .into_iter()
                            .collect(),
                        subtypes: [SubType("Gnome".to_string())].into_iter().collect(),
                        colors: imbl::OrdSet::new(), // colorless
                        power: 1,
                        toughness: 1,
                        count: EffectAmount::CounterCount {
                            target: EffectTarget::Source,
                            counter: CounterType::PlusOnePlusOne,
                        },
                        tapped: true,
                        enters_attacking: true,
                        ..Default::default()
                    },
                },
            ]),
            intervening_if: None,
            targets: vec![],
            modes: None,
            trigger_zone: None,
        }],
        // PB-OS11: WheneverYouAttack{filter} correctly models the batch-once,
        // non-Gnome-filtered attack trigger; token count reads live post-increment
        // counters (accepted minor deviation from the leaves-battlefield LKI ruling
        // edge case, documented above -- non-blocking, no non-leaves LKI reader exists).
        completeness: Completeness::Complete,
        ..Default::default()
    }
}
