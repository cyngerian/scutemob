// Greymond, Avacyn's Stalwart — {2}{W}{W}, Legendary Creature — Human Soldier 3/4
// As Greymond enters, choose two abilities from among first strike, vigilance, and lifelink.
// Humans you control have each of the chosen abilities.
// As long as you control four or more Humans, Humans you control get +2/+2.
//
// TODO: DSL gap — "As ... enters, choose from among" modal static grant is not expressible.
// No ChooseAbility ETB mechanism exists in the DSL.
// The +2/+2 static also requires a conditional static with a count threshold filter.
// Both abilities are omitted.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("greymond-avacyns-stalwart"),
        name: "Greymond, Avacyn's Stalwart".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            white: 2,
            ..Default::default()
        }),
        types: full_types(
            &[SuperType::Legendary],
            &[CardType::Creature],
            &["Human", "Soldier"],
        ),
        oracle_text: "As Greymond, Avacyn's Stalwart enters, choose two abilities from among \
                      first strike, vigilance, and lifelink.\nHumans you control have each of the \
                      chosen abilities.\nAs long as you control four or more Humans, Humans you \
                      control get +2/+2."
            .to_string(),
        power: Some(3),
        toughness: Some(4),
        abilities: vec![
            // TODO: ETB choose-two modal ability grant (no ChooseAbility ETB in DSL)
            // TODO: Conditional +2/+2 static requiring count_threshold filter
        ],
        completeness: Completeness::inert(
            "Blocked: 'As this enters, choose two abilities from among first strike, vigilance, \
             and lifelink' — no as-enters ability-choice replacement and no layer grant keyed to \
             a chosen ability set. The +2/+2 conditional static IS expressible \
             (Condition::YouControlNOrMoreWithFilter + ContinuousEffectDef.condition) and may be \
             wired — but read this first (PB-DX19, 2026-08-02): until that batch, following this \
             note would have built a SECOND instance of the OOS-SIM2-6 stack-overflow class. It \
             is safe to register now, and since PB-DX42b (scutemob-233, 2026-09-05) it is safe at \
             NO cost: the sentence that stood here until then — 'a registered static's condition \
             is evaluated from INSIDE the layer walk, where the filter test reads base \
             characteristics, so a Human created by another continuous effect's type change is \
             not counted' — is now FALSE, and so was its pointer to 'OOS-DX19-2 for the CR \
             613.8b-honest fixpoint' (CR 613.8a clause (a) confines a dependency to a single \
             layer, and this is a Layer-6 condition reading Layer-4 card types, so 613.8b never \
             governed it; the shipped repair is a LAYER-BOUNDED QUERY). A condition on a \
             continuous effect is now evaluated against characteristics resolved THROUGH the \
             layer the filter actually needs, so a Human created by a Layer-4 type change IS \
             counted, exactly as CR 613.1d requires. Rewritten rather than deleted because this \
             note was actively inviting an author to build a second member of a deviation that no \
             longer exists.",
        ),
        ..Default::default()
    }
}
