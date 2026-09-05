//! PB-DX57 (`OOS-DX28-6`): the ratchet for an in-def comment that asserts a RESOLUTION
//! MECHANISM the def's own DSL does not contain.
//!
//! # The seed
//!
//! > An in-def comment asserting a RESOLUTION MECHANISM the code does not use has no gate, and
//! > PB-DX27's sweep did not cover this shape. `sword_of_war_and_peace.rs` said *"DamagedPlayer
//! > resolves from ctx.damaged_player at resolution"* while the code declared
//! > `TargetRequirement::TargetPlayer` and read `DeclaredTarget { index: 0 }` — so in a
//! > 4-player game the CR 601.2c auto-target picker chose *a* player and the Sword could damage
//! > the wrong seat. PB-DX27 swept **blocker** notes (claims that something is unauthorable);
//! > this is the sibling class, a claim that something IS authored a particular way, and it is
//! > live-wrong in a different direction: **a blocker note that goes stale is merely
//! > pessimistic; a mechanism note that goes stale hides a live defect behind a description of
//! > the correct behaviour.** Population unmeasured.
//!
//! # The census result, stated as a result rather than as a clean bill of health
//!
//! Stage 0 measured it: **34 hits across 33 files, 22 of them real mechanism claims, and every
//! one CONFIRMED TRUE. Zero stale, zero live defects.** `sword_of_war_and_peace` appears to
//! have been the only instance of the exact shape on the axes a derivation can see, and
//! PB-DX28's repair of it is present and was re-verified. One **doc-rot-only** repair fell out
//! (`well_of_lost_dreams.rs`, `inert`, comment-only) and is taken in this batch.
//!
//! The census's derivation is not reproduced here — it needed a morphological closure over
//! 15,394 line comments and 663 `Completeness` notes to find the population without knowing
//! the answer first, which is stage-0 work. What is reproduced here is the **ratchet**: a
//! narrower, fail-closed check that holds the measured zero.
//!
//! # Why the polarity is INVERTED relative to the census's first proposal
//!
//! The census proposed keying on resolution VERBS and then subtracting the sentences that are
//! not assertions (blocker notes, rejected-alternative rationale, negations). Its own §6.2
//! then defeated that design: *"filter (d) is the load-bearing weakness and no lengthening of
//! the marker list repairs it"* — `"resolves from ctx.damaged_player, so a declared target
//! would NOT be read"* is dropped by the very filter that makes the gate usable, while stating
//! the defect in its own sentence. **Enumerating what may fire fails CLOSED; enumerating what
//! may not fire fails OPEN**, and this file takes the closed one: a small set of ASSERTIVE
//! FRAMES, each of which is a claim that the def's own code does something.
//!
//! That is PB-DX53's `/review` repair (*"enumerating what may mutate a container is unbounded
//! and fails open; enumerating the 8 READ methods is short and fails closed"*) applied to a
//! prose classifier.
//!
//! # What this ratchet does NOT claim, measured rather than hedged
//!
//! * **33.7% of the corpus's resolution-verb sentences name no identifier at all** and are
//!   structurally invisible here. The seed's own instance survives only because its author
//!   happened to write `ctx.damaged_player`; the same claim in plain English would not be seen.
//!   This is a measurement from the census, not an estimate.
//! * **A misspelled identifier is invisible**, and that is not academic: `well_of_lost_dreams`
//!   said `WhenYouGainLife` for `WheneverYouGainLife` — twice — and survived PB-DX27's sweep,
//!   PB-DX8's `completeness_deviation_scan` and the census's own Class-A pass for exactly that
//!   reason. **A claim is MORE likely to be wrong precisely when its author had the name
//!   wrong**, so this blind spot is correlated with the defect. `t4` reports near-misses rather
//!   than failing on them, because a fuzzy match is evidence and not a verdict.
//! * **The check is def-scoped, not clause-scoped.** A stale claim about a def's third ability
//!   is discharged by its first ability using the same variant for an unrelated purpose. This
//!   is PB-DX8's `/review` finding 3 one axis over, where its measured exposure was 24 defs.
//!   Closing it needs a clause-to-ability alignment nothing in the tree has.
//! * **A claim split across two sentences is invisible.** `sentences()` splits on `.`, `;` and
//!   newlines, and a hit needs the assertive frame AND the identifier in the SAME fragment, so
//!   `// Effect::Manifest is the mechanism here. It resolves from the declared target.` is
//!   green while the identical claim as one sentence fires. Found by the `/review`; a fifth
//!   bound, and it was not in this list.
//! * **`ASSERTIVE_FRAMES` is hand-typed and therefore a floor**, which is the correct polarity
//!   (enumerating what may fire fails CLOSED) but is still a bound: `dispatch`, `calls` and
//!   `gets applied by` are ordinary assertive verbs and none is listed. Stated in the const's
//!   own doc and now here, because a bound a reader has to go looking for is not disclosed.
//! * The corpus contains **zero** `/* */` comments, so the block-comment arm of every extractor
//!   here — and of the three that already existed — has never run against real input. Stated
//!   because *a scanner whose only evidence of correctness is that it has never had to run* is
//!   `OOS-DX32-6`'s shape. `t5` exercises it on synthetic input instead.

use crate::completeness_deviation_scan::{
    block_comment_bodies, completeness_note_bodies, read_def_sources,
};
use crate::pb_dx57_declared_source::{
    declared_enums_in, CARD_DEFINITION_RS, CONTINUOUS_EFFECT_RS, GAME_OBJECT_RS, STATE_TYPES_RS,
};
use std::collections::{BTreeMap, BTreeSet};

/// Non-vacuity floors for the declared dictionary. Measured **58 enums / 909 variants**.
const MIN_DICT_ENUMS: usize = 50;
/// See [`MIN_DICT_ENUMS`].
const MIN_DICT_VARIANTS: usize = 800;

/// The assertive frames. Each says *"this def's code does X"*, so the claim is checkable
/// against the def's own stripped code.
///
/// Deliberately SHORT and deliberately assertive-only. A frame added here can only make the
/// gate fire more; a frame that belongs here and is missing makes it fire less, which is the
/// stated recall bound above rather than a silent failure. Matched case-insensitively against
/// a sentence.
const ASSERTIVE_FRAMES: &[&str] = &[
    // ── The original eleven ──────────────────────────────────────────────────
    "resolves from",
    "resolves via",
    "resolves through",
    "is resolved from",
    "reads ",
    "is read from",
    "is substituted",
    "substituted into",
    "is taken from",
    "comes from",
    "is looked up",
    // ── Added after the adversarial pass, which defeated the eleven above with
    //    EIGHT plain-English assertive verbs, none of them exotic ─────────────
    //
    // Widening is the CORRECT repair for a fail-closed design and the reason the polarity was
    // inverted in the first place: a frame that belongs here and is missing makes the gate
    // fire LESS, which is the stated recall bound, whereas an over-wide *exclusion* list in the
    // other polarity would make it fail OPEN. So a defeat by "you missed a verb" is answered by
    // adding the verb, and each addition is monotone.
    //
    // **The sharpest of the eight was `snapshots`** — the verb in this file's own recorded
    // offender `chandra_flamecaller` (*"Effect::WheelHand snapshots the pre-disposal hand
    // size"*), which the gate caught there only through a DIFFERENT clause in the same comment.
    // A gate that already holds a row it could not have found is a gate whose frame list is
    // narrower than its own evidence.
    "handles",
    "supplies",
    "does the work",
    "drives",
    "implements",
    "performs",
    "is used to resolve",
    "snapshots",
];

/// Prose that carries no identifier the DECLARATION knows is not a claim this gate can check.
/// These are the enum-declaring files whose variants form the dictionary. `card-types` only:
/// an engine-internal identifier named in a card-def comment is describing plumbing the def
/// cannot contain, which is the largest false-positive shape the census measured.
const DICTIONARY_FILES: &[&str] = &[
    CARD_DEFINITION_RS,
    STATE_TYPES_RS,
    CONTINUOUS_EFFECT_RS,
    GAME_OBJECT_RS,
];

/// `Enum -> variants`, parsed from the declarations at run time.
///
/// The VARIANT NAMES are never learned from corpus usage — a dictionary derived from the thing
/// being checked cannot disagree with it (PB-DX8). The set of ENUMS is then narrowed to the
/// **authorable** ones: an enum no card def's code ever names is engine plumbing, and a comment
/// naming one of its variants is describing the engine rather than making a claim about this
/// def's DSL. That narrowing is itself DERIVED (does any def's stripped code contain
/// `<Enum>::`?), not a hand-written exclusion list, because a hand-written list of
/// non-authorable enums is the very shape this batch exists to remove.
///
/// Measured effect: it removes `TriggerEvent` — `elenda_the_dusk_rose`'s note correctly
/// explains that `EffectAmount::SourcePowerAtLastKnownInformation` *"reads the LKI power
/// captured when the SelfDies trigger was queued"*, and `TriggerEvent::SelfDies` is a runtime
/// type a card def structurally cannot contain. Excluding it by mechanism keeps the sentence,
/// which is TRUE and worth having.
fn declared_dictionary() -> BTreeMap<String, BTreeSet<String>> {
    let mut all: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for f in DICTIONARY_FILES {
        for (name, variants) in declared_enums_in(f) {
            all.entry(name).or_default().extend(variants);
        }
    }
    let corpus: Vec<String> = read_def_sources()
        .into_iter()
        .map(|(_, src)| code_surface(&src))
        .collect();
    let out: BTreeMap<String, BTreeSet<String>> = all
        .into_iter()
        .filter(|(name, _)| {
            let needle = format!("{name}::");
            corpus.iter().any(|c| c.contains(&needle))
        })
        .collect();
    assert!(
        !out.is_empty(),
        "no declared enum is authorable, so the dictionary is empty and every 'is this a real \
         identifier' question answers NO -- the ratchet would report zero offenders forever"
    );
    out
}

/// Author prose, CASE PRESERVED.
///
/// `completeness_deviation_scan::author_prose` lowercases (its needles are English), which
/// destroys the identifier case this gate matches on — so the line-comment part is done here
/// and the two hard parts are REUSED rather than re-implemented: `completeness_note_bodies`
/// joins `\`-continuations (a naive extractor silently truncates the **89.4%** of notes that
/// span lines — `OOS-DX35`) and `block_comment_bodies` handles `/* */`. *A needle set and the
/// surface it is matched against are two halves of one instrument*, so the surface is shared.
fn author_prose_cased(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        if let Some(idx) = line.find("//") {
            out.push_str(&line[idx + 2..]);
            out.push('\n');
        }
    }
    for body in block_comment_bodies(src) {
        out.push_str(&body);
        out.push('\n');
    }
    for note in completeness_note_bodies(src) {
        out.push_str(&note);
        out.push('\n');
    }
    out
}

/// The def's CODE surface: comments and string literals removed.
///
/// Both, and the string literals matter more than they look: a `Completeness::partial("...
/// Effect::Foo ...")` note is a Rust STRING LITERAL, not a comment, and `OOS-DX53-2` records a
/// census that read **5** where the truth was **4** because a `Debug` render counted compiled
/// prose as a declaration. A gate whose whole subject is *"the comment says X, does the code
/// contain X"* cannot let the comment be part of the code surface.
fn code_surface(src: &str) -> String {
    let b = src.as_bytes();
    let mut out = String::with_capacity(src.len());
    let mut i = 0usize;
    while i < b.len() {
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i = (i + 2).min(b.len());
            continue;
        }
        if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i = (i + 1).min(b.len());
            out.push(' ');
            continue;
        }
        out.push(b[i] as char);
        i += 1;
    }
    out
}

/// Split prose into sentences. `.`/`;` boundaries, plus newlines, because an author's comment
/// is often one clause per line with no terminator.
fn sentences(prose: &str) -> Vec<String> {
    prose
        .split(['.', ';', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Declared variant names that are ALSO ordinary English or ordinary Magic card text, and so
/// fire on prose that is making no mechanism claim. Removed from the bare axis only; a
/// QUALIFIED `Enum::Variant` mention of any of these still counts.
///
/// Kept deliberately short. Each entry weakens the gate, so an entry is a cost, not a
/// convenience — and `m6` re-checks that every entry is still a real declared variant, because
/// an exclusion whose subject no longer exists is a comment (`OOS-DX52-1`).
const AMBIGUOUS_BARE: &[&str] = &[];

/// Every identifier in a sentence that the DECLARATION knows, on **two** axes.
///
/// # Why two, and why the second one is the load-bearing half
///
/// The obvious axis is the qualified `Enum::Variant` token. **The first draft of this gate had
/// only that axis, and it did not fire on the seed's own pre-repair sentence** — which is what
/// `m4` is for, and it caught it on the first run. The sentence
///
/// > *"DamagedPlayer resolves from ctx.damaged\_player at resolution"*
///
/// **contains no `::` at all.** The stage-0 census predicted this in as many words (*"a
/// Class-A-only derivation would have missed the seed's own instance"*), and the prediction
/// was reproduced here by execution rather than taken on trust. So the BARE identifier axis
/// carries the known positive, and a gate built on qualified tokens alone would have been a
/// gate that could not catch the one instance the class is known to have had.
///
/// The bare axis is resolved against the declared dictionary — a bare CamelCase word counts
/// only if some `card-types` enum declares a variant of that name — so it is not a general
/// CamelCase scan. `AMBIGUOUS_BARE` removes the handful of variant names that are also
/// ordinary English or ordinary card text, because those fire on prose that is not a
/// mechanism claim at all.
fn declared_identifiers_in(
    sentence: &str,
    dict: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    // Axis 2 (bare): a CamelCase word that some declared enum names as a variant.
    let mut word = String::new();
    let push_word = |w: &mut String, out: &mut BTreeSet<String>| {
        if w.len() >= 4
            && w.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && w.chars().skip(1).any(|c| c.is_ascii_uppercase())
            && !AMBIGUOUS_BARE.contains(&w.as_str())
        {
            for (e, vs) in dict.iter() {
                if vs.contains(w.as_str()) {
                    out.insert(format!("{e}::{w}"));
                    break;
                }
            }
        }
        w.clear();
    };
    for ch in sentence.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
        } else {
            push_word(&mut word, &mut out);
        }
    }
    push_word(&mut word, &mut out);
    let bytes: Vec<char> = sentence.chars().collect();
    let mut i = 0usize;
    while i + 1 < bytes.len() {
        if bytes[i] == ':' && bytes[i + 1] == ':' {
            let mut a = i;
            while a > 0 && (bytes[a - 1].is_ascii_alphanumeric() || bytes[a - 1] == '_') {
                a -= 1;
            }
            let lhs: String = bytes[a..i].iter().collect();
            let mut b = i + 2;
            while b < bytes.len() && (bytes[b].is_ascii_alphanumeric() || bytes[b] == '_') {
                b += 1;
            }
            let rhs: String = bytes[i + 2..b].iter().collect();
            if let Some(vs) = dict.get(&lhs) {
                if vs.contains(&rhs) {
                    out.insert(format!("{lhs}::{rhs}"));
                }
            }
            i = b.max(i + 2);
            continue;
        }
        i += 1;
    }
    out
}

/// Does `hay` contain `needle` as a WHOLE TOKEN?
///
/// **Load-bearing, and the first draft used `contains`.** The declared dictionary is full of
/// prefix pairs — `AddMana` / `AddManaAnyColor` / `AddManaScaled`, `AddCounter` /
/// `AddCounterAmount`, `Controller` / `ControllerOf`, `TargetCreature` /
/// `TargetCreatureWithFilter` — so a substring match lets a def DISCHARGE a stale claim about
/// `Effect::AddMana` with its own unrelated `Effect::AddManaAnyColor`. Proven by the `/review`:
/// a def declaring `AddManaAnyColor` and commenting *"Effect::AddMana resolves from the mana
/// ability at resolution"* went GREEN, while the control (`Effect::Manifest`, which has no
/// superstring) fired correctly. `has_token` already existed in this batch's sibling gate, in
/// the same commit.
fn has_token(hay: &str, needle: &str) -> bool {
    let is_ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut from = 0usize;
    while let Some(rel) = hay[from..].find(needle) {
        let at = from + rel;
        let before_ok = at == 0 || !hay[..at].chars().next_back().is_some_and(is_ident);
        let after = at + needle.len();
        let after_ok = after >= hay.len() || !hay[after..].chars().next().is_some_and(is_ident);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// The predicate `offenders()` applies to ONE (prose, code) pair.
///
/// **Extracted so `m3` and `m4` call it instead of re-implementing it.** `m4` is the
/// known-positive replay for the seed's own sentence, and the `/review` pointed out that it had
/// hand-inlined the four-step pipeline: repair `offenders()` (as Issue 4 did, from `contains`
/// to `has_token`) and `m4` keeps passing on the OLD rule, so the seed replay silently stops
/// testing what ships. That is the batch's own "a gate on a COPY" finding, one file over.
fn mechanism_claim_offenders(
    prose: &str,
    code: &str,
    dict: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for sentence in sentences(prose) {
        let lower = sentence.to_lowercase();
        if !ASSERTIVE_FRAMES.iter().any(|f| lower.contains(f)) {
            continue;
        }
        for id in declared_identifiers_in(&sentence, dict) {
            let (_, variant) = id.split_once("::").expect("built with ::");
            if !has_token(code, variant) {
                out.push((id.clone(), sentence.clone()));
            }
        }
    }
    out
}

/// `(def file stem, the identifier, the sentence)` for every assertive mechanism claim whose
/// named identifier does not occur in that def's own code.
fn offenders() -> Vec<(String, String, String)> {
    let dict = declared_dictionary();
    let mut out = Vec::new();
    for (name, src) in read_def_sources() {
        let prose = author_prose_cased(&src);
        let code = code_surface(&src);
        for (id, sentence) in mechanism_claim_offenders(&prose, &code, &dict) {
            out.push((name.clone(), id, sentence));
        }
    }
    out.sort();
    out
}

/// The measured population, frozen. Ratcheted in BOTH directions: a new offender is a new
/// instance of `OOS-DX28-6`; a vanished one means the claim was repaired (good — record it and
/// shrink the list) or that the SCANNER narrowed (bad, and indistinguishable from the first
/// without looking).
const RECORDED_OFFENDERS: &[(&str, &str, &str)] = &[
    // ── Shape 1: REJECTED-ALTERNATIVE rationale ──────────────────────────────
    // The sentence describes what a construction the def DOES NOT USE would do. It is
    // excellent documentation and the census (§6.3) is explicit that a gate which makes an
    // author delete a true sentence to go green has stopped measuring -- PB-DX54's `r3`
    // finding stated inside this file's own subject matter. Recorded, not filtered: a
    // NEGATION/hypothetical filter is what the census's own §6.2 defeats
    // ("filter (d) is the load-bearing weakness and no lengthening of the marker list repairs
    // it"), because a stale claim can carry a negation word in a neighbouring clause while
    // asserting the defect in its own.
    (
        "chandra_flamecaller",
        "Effect::DiscardCards",
        "rejected-alternative rationale: 'a naive DiscardCards{HandSize}+DrawCards{HandSize} \
         reads 0 after the hand is already emptied' explains why the def uses Effect::WheelHand \
         INSTEAD. Re-verified TRUE at HEAD (effects/mod.rs's HandSize amount is evaluated after \
         the discard, and the def really does declare WheelHand).",
    ),
    (
        "chandra_flamecaller",
        "EffectAmount::HandSize",
        "same sentence, second identifier. Same adjudication.",
    ),
    // ── Shape 2: the comment names the RUNTIME identifier the DECLARED one lowers to ──
    // Not a defect and not noise: it is how a reader traces a def into the engine, and this
    // one is CONFIRMED TRUE by reading the lowering. `elenda_the_dusk_rose` declares
    // `TriggerCondition::WhenDies`, which `replay_harness.rs:2677` lowers to
    // `trigger_on: TriggerEvent::SelfDies` -- so the sentence *"EffectAmount::
    // SourcePowerAtLastKnownInformation reads the LKI power captured when the SelfDies trigger
    // was queued"* is accurate, and the amount it names IS in the def's code
    // (`elenda_the_dusk_rose.rs:66`).
    //
    // Note that `TriggerEvent` is NOT removed by the authorability filter, and correctly so:
    // `basri_ket.rs:78` and `ajani_sleeper_agent.rs:69` construct `TriggerEvent` variants in
    // real emblem trigger specs, so the enum genuinely is authorable. The first draft of this
    // file predicted the filter would remove it; the corpus refuted that, and the refutation is
    // recorded rather than the prediction quietly deleted.
    (
        "elenda_the_dusk_rose",
        "TriggerEvent::SelfDies",
        "CONFIRMED TRUE: the comment names the runtime TriggerEvent that the def's declared \
         TriggerCondition::WhenDies lowers to (replay_harness.rs:2677), and the \
         EffectAmount::SourcePowerAtLastKnownInformation it describes is declared at \
         elenda_the_dusk_rose.rs:66. Verified by reading the lowering site, not inferred.",
    ),
    // ── Shape 4: a claim about what the ENGINE handles, in a def that authors nothing ──
    // Surfaced only after the adversarial pass widened the frame list (`handles` was one of the
    // eight verbs that defeated the first eleven), which is the widening working: one new hit
    // on 1,803 defs, and it is real prose worth keeping. `tectonic_giant` declares
    // `abilities: vec![]` DELIBERATELY -- W6 policy, since authoring the modal ability either
    // way produces wrong game state -- and explains why by naming what the engine CAN do. The
    // identifier is correctly absent from the def's own source because the def has no source
    // to put it in.
    //
    // Distinct from shape 2: elenda's note names the runtime identifier its own DECLARED
    // identifier lowers to; this one names an identifier the ENGINE handles, in a def that
    // declares nothing at all.
    (
        "tectonic_giant",
        "AbilityDefinition::Triggered",
        "CONFIRMED TRUE and correctly absent: the note says 'resolution.rs handles \
         AbilityDefinition::Triggered { modes, .. }' -- a claim about the ENGINE, in a def whose \
         `abilities` is deliberately EMPTY (mode 1 has no DSL representation, and W6 policy is \
         that a partial modal ability is worse than an absent one). There is no def source for \
         the identifier to appear in, which is the point of the note.",
    ),
    // ── Shape 2 again, and it was HIDDEN by a substring match until the `/review` ──
    // `olivias_wrath` declares `LayerModification::ModifyBothDynamic` and its note says
    // *"ModifyBothDynamic is substituted into ModifyBoth(-X)"* -- naming the runtime shape the
    // declared one becomes, exactly like `elenda_the_dusk_rose` above. It appeared only when
    // the code-surface match was tightened from `contains` to a whole-token match: the def's
    // own `ModifyBothDynamic` CONTAINS the string `ModifyBoth`, so the claim was discharging
    // itself. **That is the prefix-pair blindness the `/review` found, caught on its first run
    // after the fix**, and it is why this row exists rather than a quieter gate.
    (
        "olivias_wrath",
        "LayerModification::ModifyBoth",
        "CONFIRMED TRUE: the def declares ModifyBothDynamic (olivias_wrath.rs:28) and the note \
         names the shape the engine substitutes it into. `rules/layers.rs:2696-2709` documents \
         exactly that substitution and says the plain-ModifyBoth arm is reached only from it. \
         Verified by reading the engine site, not inferred -- and this row was INVISIBLE while \
         the code surface was matched by substring, because ModifyBothDynamic contains \
         ModifyBoth.",
    ),
    // ── Shape 3: PROSPECTIVE rationale (a claim about a rewire not yet made) ──
    (
        "fecundity",
        "PlayerTarget::ControllerOf",
        "prospective: 'Residual AFTER REWIRE: ... ControllerOf reads the graveyard object whose \
         controller was reset to owner (CR 603.10a)' is a claim about what the def WOULD do if \
         rewired, inside a Completeness::partial note that says so. The def declares \
         PlayerTarget::Controller today, which its own note calls an approximation. Re-verified \
         at HEAD.",
    ),
];

// ── The gate ─────────────────────────────────────────────────────────────────

#[test]
fn m1_no_def_asserts_a_resolution_mechanism_its_own_code_does_not_contain() {
    let live = offenders();
    let recorded: BTreeSet<(String, String)> = RECORDED_OFFENDERS
        .iter()
        .map(|(a, b, _)| (a.to_string(), b.to_string()))
        .collect();
    let live_keys: BTreeSet<(String, String)> = live
        .iter()
        .map(|(a, b, _)| (a.clone(), b.clone()))
        .collect();

    let new: Vec<&(String, String, String)> = live
        .iter()
        .filter(|(a, b, _)| !recorded.contains(&(a.clone(), b.clone())))
        .collect();
    assert!(
        new.is_empty(),
        "OOS-DX28-6: {} card def(s) carry a comment ASSERTING that the def's code uses a \
         mechanism the def's own (comment-stripped, string-stripped) source does not contain:\n\
         {new:#?}\n\
         This is the `sword_of_war_and_peace` shape: a blocker note that goes stale is merely \
         pessimistic, a MECHANISM note that goes stale hides a live defect behind a description \
         of the correct behaviour. Read the def, decide whether the COMMENT or the CODE is \
         wrong, and fix that one -- do not add the identifier to the code to make this green, \
         and do not reword the comment to dodge the frame list, which is a gate you edit prose \
         to satisfy and has stopped measuring.",
        new.len()
    );

    let gone: Vec<&(String, String)> = recorded.difference(&live_keys).collect();
    assert!(
        gone.is_empty(),
        "recorded offender(s) {gone:?} no longer fire. If the claim was repaired, delete the \
         row and say so; if nothing was repaired, the SCANNER has narrowed and the ratchet is \
         now blind -- the two are indistinguishable from the count alone, which is why this \
         assertion exists."
    );
}

// ── Non-vacuity: every one of these executes rather than asserting a constant ─

/// The dictionary is real. A dictionary that came back empty would make
/// `declared_identifiers_in` return nothing for every sentence and `m1` would report zero
/// offenders forever — the seed's own failure mode entering through its own fix.
#[test]
fn m2_the_declared_dictionary_is_not_vacuous() {
    let dict = declared_dictionary();
    let variants: usize = dict.values().map(|v| v.len()).sum();
    // The floors are INTERPOLATED into the message rather than restated in prose. The first
    // draft asserted `>= 50 && >= 800` under a message reading *"(floor 60 / 700)"* — so a
    // maintainer who corrected the code to match the message would have reddened the enum axis
    // and LOOSENED the variant axis. A number written twice is a number that can disagree with
    // itself, which is this batch's whole subject one scale down.
    assert!(
        dict.len() >= MIN_DICT_ENUMS && variants >= MIN_DICT_VARIANTS,
        "declared dictionary is {} enums / {variants} variants (floors {MIN_DICT_ENUMS} / \
         {MIN_DICT_VARIANTS}, measured at 58 / 909 when PB-DX57 wrote them). Raise-only: a \
         shrinking dictionary silently narrows m1 to nothing.",
        dict.len()
    );
    // Spot-check the two identifiers this seed's own history turns on.
    assert!(dict["TargetRequirement"].contains("TargetPlayer"));
    assert!(dict["TriggerCondition"].contains("WheneverYouGainLife"));
}

/// The scan reaches the corpus and the frames fire on it. A frame list that matched NOTHING
/// would make `m1` vacuous while green.
#[test]
fn m3_the_frames_match_real_prose_in_the_real_corpus() {
    let defs = read_def_sources();
    assert!(
        defs.len() >= 1_700,
        "the def sweep read only {} files",
        defs.len()
    );
    let mut matched = 0usize;
    let mut with_identifier = 0usize;
    let dict = declared_dictionary();
    for (_, src) in &defs {
        for s in sentences(&author_prose_cased(src)) {
            let lower = s.to_lowercase();
            if ASSERTIVE_FRAMES.iter().any(|f| lower.contains(f)) {
                matched += 1;
                if !declared_identifiers_in(&s, &dict).is_empty() {
                    with_identifier += 1;
                }
            }
        }
    }
    assert!(
        matched >= 70 && with_identifier >= 28,
        "the assertive frames matched {matched} sentence(s), {with_identifier} of them naming a \
         declared identifier (floors 70 / 28, measured at 91 / 35 after the adversarial pass \
         widened the frame list from 11 frames to 19; it read 56 / 23 before). Below these the \
         gate is \
         measuring nothing. \
         NOTE the ratio: the gap between the two numbers IS this gate's recall bound -- a \
         mechanism claim written in plain English is invisible to it, which the stage-0 census \
         measured at 33.7% of resolution-verb sentences."
    );
    println!(
        "PB-DX57 / OOS-DX28-6 — {matched} assertive-frame sentences across {} defs; \
         {with_identifier} name a declared identifier; live offenders: {}",
        defs.len(),
        offenders().len()
    );
}

/// The gate fires on a planted instance, and it fires on the SEED'S OWN pre-repair sentence.
///
/// This is the known-positive replay: `sword_of_war_and_peace.rs` said *"DamagedPlayer resolves
/// from ctx.damaged_player at resolution"* while the code read `DeclaredTarget { index: 0 }`.
/// The frame is `resolves from`; the identifier is `PlayerTarget::DamagedPlayer`. Run against
/// synthetic sources rather than against the corpus, so the assertion cannot be satisfied by
/// the corpus happening to contain the shape — and cannot silently stop meaning anything the
/// day the corpus changes.
#[test]
fn m4_the_gate_fires_on_the_seeds_own_pre_repair_sentence() {
    let dict = declared_dictionary();

    // The seed's sentence, verbatim in substance, against code that does NOT name the variant.
    let stale = "// DamagedPlayer resolves from ctx.damaged_player at resolution.\n\
                 pub fn card() { targets: vec![TargetRequirement::TargetPlayer] }";
    let prose = author_prose_cased(stale);
    let code = code_surface(stale);
    let hit = !mechanism_claim_offenders(&prose, &code, &dict).is_empty();
    assert!(
        hit,
        "the gate does not fire on the seed's own pre-repair sentence, so it would not have \
         caught the one instance the class is known to have had"
    );

    // And it must NOT fire once the code really does contain the mechanism (the repaired
    // shape), or it is an unconditional failure rather than a check.
    let repaired = "// DamagedPlayer resolves from ctx.damaged_player at resolution.\n\
                    pub fn card() { target: PlayerTarget::DamagedPlayer }";
    let rprose = author_prose_cased(repaired);
    let rcode = code_surface(repaired);
    let rhit = !mechanism_claim_offenders(&rprose, &rcode, &dict).is_empty();
    assert!(
        !rhit,
        "the gate fires on the REPAIRED shape too, so it is not discriminating the claim from \
         the code -- it would force authors to delete true sentences to go green, which is a \
         gate that has stopped measuring"
    );
}

/// The two surfaces are genuinely separated: a mechanism named ONLY inside a `Completeness`
/// note (a string literal, not a comment) must count as PROSE and must NOT count as CODE.
///
/// `OOS-DX32-6` in both directions at once, and the block-comment arm exercised on synthetic
/// input because the corpus carries zero `/* */` comments and so cannot test it.
#[test]
fn m5_prose_and_code_surfaces_are_separated_in_both_directions() {
    let src = "completeness: Completeness::partial(\"needs Effect::Manifest\"),\n\
               /* block: PlayerTarget::DamagedPlayer resolves from ctx.damaged_player */\n\
               let x = Effect::DrawCards;";
    let prose = author_prose_cased(src);
    let code = code_surface(src);
    assert!(
        prose.contains("Effect::Manifest"),
        "a mechanism named only in a Completeness note is not reaching the prose surface"
    );
    assert!(
        prose.contains("PlayerTarget::DamagedPlayer"),
        "a block comment is not reaching the prose surface -- the corpus has zero of them, so \
         this arm has no real input and would rot untested"
    );
    assert!(
        !code.contains("Manifest"),
        "a string literal is reaching the CODE surface, so a def could discharge its own \
         mechanism claim with the note that MAKES the claim"
    );
    assert!(
        !code.contains("DamagedPlayer"),
        "a block comment is reaching the CODE surface"
    );
    assert!(
        code.contains("DrawCards"),
        "real code is being stripped, which would make every def look like it contains nothing"
    );
}

/// `OOS-DX52-1`: **an allowlist whose reason is not checked is a comment.** That seed was filed
/// when `completeness_deviation_scan`'s `RECORDED_BASELINE` was found quoting a fragment its
/// own def no longer contained — the entry kept passing because the def still matched the same
/// needles for a different reason, and nothing in the tree checked the quoted text.
///
/// So each recorded entry is re-checked here on the two axes that can rot: the def must still
/// exist, and the identifier must still be a DECLARED identifier. `m1` already carries the
/// third axis (the entry must still FIRE), which is the one that catches a repair.
///
/// `AMBIGUOUS_BARE` is **empty** at the time of writing, so its half of this test is vacuous
/// today and says so rather than reading as coverage. It exists because the first draft of
/// this gate expected to need exclusions and did not — the bare axis produced four hits on
/// 1,803 defs, all adjudicated, none needing a word-level exclusion.
#[test]
fn m6_every_recorded_entry_is_still_live_on_its_own_terms() {
    let dict = declared_dictionary();
    let defs: BTreeSet<String> = read_def_sources().into_iter().map(|(n, _)| n).collect();

    for (def, id, reason) in RECORDED_OFFENDERS {
        assert!(
            defs.contains(*def),
            "recorded offender names def `{def}`, which is no longer in the corpus. Delete the \
             row rather than leaving an entry that can never be re-adjudicated."
        );
        let (e, v) = id.split_once("::").expect("recorded ids are qualified");
        assert!(
            dict.get(e).is_some_and(|vs| vs.contains(v)),
            "recorded offender `{id}` (on {def}) is no longer a declared identifier of an \
             authorable enum. Either it was renamed -- in which case the def's comment is now \
             wrong in a NEW way and needs re-reading, not a silent allowlist entry -- or the \
             enum stopped being authorable and the row is dead."
        );
        assert!(
            reason.len() > 40,
            "recorded offender ({def}, {id}) carries a reason of {} chars. An entry whose \
             adjudication is not written down is an allowlist entry, which is what OOS-DX52-1 \
             is about.",
            reason.len()
        );
    }

    for w in AMBIGUOUS_BARE {
        assert!(
            dict.values().any(|vs| vs.contains(*w)),
            "AMBIGUOUS_BARE names `{w}`, which is not a declared variant of any authorable \
             enum -- so the exclusion removes nothing and is dead weight that reads as coverage."
        );
    }
    // NOTE: an earlier draft closed this test with `assert!(AMBIGUOUS_BARE.is_empty() ||
    // !AMBIGUOUS_BARE.is_empty(), ..)`. That is `A || !A` -- a compile-time tautology, and it
    // is `t9_fingerprints_match_their_structs`'s ORIGINAL defect (`intersection.is_none() ||
    // len != len`) reproduced by hand inside the batch whose subject is assertions that cannot
    // fail. Deleted rather than reworded: the loop above IS the check, and a line that cannot
    // fail adds nothing but the appearance of one.
    println!(
        "PB-DX57 / OOS-DX28-6 — {} recorded adjudications re-checked; AMBIGUOUS_BARE holds {} \
         entries (empty today, so that half of this test is VACUOUS and is stated as such)",
        RECORDED_OFFENDERS.len(),
        AMBIGUOUS_BARE.len()
    );
}
