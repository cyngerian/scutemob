// Serpent's Soul-Jar — {2}{B}, Artifact
// Whenever an Elf you control dies, exile it.
// {T}, Pay 2 life: Until end of turn, you may cast a creature spell from among cards
//   exiled with this artifact.
//
// ENGINE-BLOCKED: Multiple DSL gaps:
// 1. "Whenever an Elf you control dies, exile it" requires a death trigger that
//    exiles the dying creature itself (EffectTarget::TriggeringCreature), plus the
//    exile must be tagged as "exiled with this artifact" for zone tracking.
//    No "exile dying creature" effect with artifact-tagged exile exists.
// 2. "{T}, Pay 2 life: ... cast a creature spell from among cards exiled with this
//    artifact" requires cast-from-exile permission gated on a specific exile source.
//    No such cast-from-exile permission primitive exists in the DSL.
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("serpents-soul-jar"),
        name: "Serpent's Soul-Jar".to_string(),
        mana_cost: Some(ManaCost {
            generic: 2,
            black: 1,
            ..Default::default()
        }),
        types: types(&[CardType::Artifact]),
        oracle_text: "Whenever an Elf you control dies, exile it.\n{T}, Pay 2 life: Until end of \
                      turn, you may cast a creature spell from among cards exiled with this \
                      artifact."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "Multiple DSL gaps: 1. 'Whenever an Elf you control dies, exile it' requires a death \
             trigger that exiles the dying...",
        ),
        ..Default::default()
    }
}
