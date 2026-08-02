// Cyber Conversion — {U}{U}, Instant
// Turn target creature face down. It's a 2/2 Cyberman artifact creature.
//
// TODO (CARDS-2, scutemob-181): DSL gap. The engine's face-down machinery
// (`FaceDownKind::{Morph,Megamorph,Disguise,Manifest,Cloak}`) only covers a card
// entering the battlefield face down as part of casting it or putting it into play
// (Morph/Megamorph/Disguise casts; Manifest/Cloak from library). It has no primitive
// for a target-based spell effect that turns an ALREADY-ON-BATTLEFIELD permanent
// (of any owner) face down in place — no zone change occurs, `status.face_down`
// flips on the same object. There is also no `FaceDownKind::Cyberman` (or
// equivalent) variant: this card's face-down state is a 2/2 *artifact* creature
// (CR 707.1a's default face-down characteristics are colorless creature 2/2 with no
// text/name/types, and this card layers "artifact" on top per the ruling that the
// Cyberman-ness ends the moment the permanent is turned face up). Missing
// primitives: (a) an `Effect::TurnPermanentFaceDown { target, .. }` (or similar) that
// flips `status.face_down` on a targeted permanent without moving zones, and (b) a
// `FaceDownKind` variant (or a characteristics override) carrying the extra
// "artifact" type while face-down this way. Per W5 policy, leaving abilities empty
// rather than authoring a wrong/partial effect (there is no draw, and no
// until-end-of-turn duration — turning a creature face down is permanent until
// something turns it face up).
use crate::cards::helpers::*;

pub fn card() -> CardDefinition {
    CardDefinition {
        card_id: cid("cyber-conversion"),
        name: "Cyber Conversion".to_string(),
        mana_cost: Some(ManaCost {
            blue: 2,
            ..Default::default()
        }),
        types: types(&[CardType::Instant]),
        oracle_text: "Turn target creature face down. It's a 2/2 Cyberman artifact creature."
            .to_string(),
        abilities: vec![],
        completeness: Completeness::inert(
            "def was authored against text this card does not have. Real oracle is {U}{U} 'Turn \
             target creature face down. It's a 2/2 Cyberman artifact creature.' The def instead \
             applied a temporary Layer-4 type change (add Artifact until end of turn) plus a draw \
             a card — a spell that does not exist. DSL gap: no Effect turns an \
             already-on-battlefield target creature face down in place \
             (Morph/Megamorph/Disguise/Manifest/Cloak are all enter-the-battlefield-face-down \
             mechanisms, not target-based spell effects), and no FaceDownKind variant carries the \
             'plus artifact' Cyberman characteristics. Requires a new Effect primitive and \
             FaceDownKind variant (or characteristics override).",
        ),
        ..Default::default()
    }
}
