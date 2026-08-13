//! PB-DX8 / **OOS-DP10-9** — the oracle-text-vs-DSL cross-check.
//!
//! ## The hole this closes
//!
//! Every one of `decision_gate.rs`'s 22 `ROWS` is a predicate over a **DSL variant name**. That
//! makes the whole instrument blind to one class by construction: a card whose printed *"you may
//! X. If you do, Y"* or *"choose one —"* was **dropped at authoring time** carries no variant to
//! match, hits zero rows, and passes `T4`/`T6` forever. `docs/audits/decision-point-audit.md`
//! §3.1 states the severity plainly, and it is the right way round: a **recorded auto-choice is at
//! least a legal outcome; a dropped "may" is not** — the engine performs an action the printed
//! card says the player may decline.
//!
//! Smuggler's Copter is the worked example (`OOS-DP10-8`): printed *"you may draw a card. If you
//! do, discard a card"*, authored as a bare `Effect::Sequence(DrawCards, DiscardCards)` on both
//! the attack and the block trigger. It reached `decision_gate.rs`'s `BASELINE` only via the
//! **incidental** `Effect::DiscardCards` inside that sequence — i.e. for a reason unrelated to the
//! defect. This file is the instrument that sees the defect itself; `t_smugglers_copter_is_in_the_
//! measured_population` pins that it does.
//!
//! ## What is checked
//!
//! For each **channel** (below), a card definition is compared along two independent axes:
//!
//! * the **oracle axis** — does the printed text grant a choice of this channel's kind?
//! * the **DSL axis** — does the serialized `CardDefinition` carry any construct capable of
//!   *expressing* a choice of this channel's kind?
//!
//! A def that is oracle-positive and DSL-negative has, by construction, dropped the choice. That
//! pairing is the whole gate.
//!
//! ## Both vocabularies are DERIVED, and the derivations are stated
//!
//! The acceptance criterion forbids a hand-written list on either axis, because a hand list is
//! `OOS-CARDS2-7`'s own defect recurring ("derive the category from the thing being checked, not
//! from the checker" — `memory/primitives/seed-rerank-2026-08-02.md` §2.6).
//!
//! **Oracle axis (morphological closure, [`decision_word_closure`]).** Start from the three
//! markers the brief names — `may`, `choose`, `up to`. For each single-word marker, take its
//! **first three characters** as a stem and admit every whole word in the corpus's own oracle text
//! that begins with it; the phrase marker `up to` matches as a phrase. The closure is computed
//! from `all_cards()` at run time, not transcribed, so a newly-authored inflection joins it
//! automatically — and [`t_decision_word_closure_is_pinned`] pins the current members so that
//! joining is *visible* rather than silent. Measured at 2026-08-12: `may` → {`may`} (plus the one
//! excluded false positive below); `cho` → {`choice`, `choose`, `chooses`, `chosen`}.
//!
//! **DSL axis (identifier stemming, [`dsl_elements_for_stem`]).** Walk every serialized
//! `CardDefinition` in `all_cards()`, collect every object **key** and every bare **variant
//! string** (the `decision_site_walk` unit-variant lesson — a unit variant serializes as a bare
//! JSON string, not an object key, and a key-only walk reports zero for it), tokenize each
//! identifier on camelCase/snake_case boundaries, and keep the ones carrying a token that begins
//! with the **same stem** the oracle axis uses. One vocabulary, two surfaces. Measured:
//! `may` → {`MayPayOrElse`, `MayPayThenEffect`}; `cho` → 20 elements (`ChooseColor`,
//! `ChooseCreatureType`, `ChooseABackground`, the `*ChosenType*`/`*ChosenColor*` family, …);
//! `up to` → {`UpToN`}.
//!
//! ## Suppressions are explicit, reasoned entries — never silent needle tuning
//!
//! Some DSL constructs express optionality **structurally** rather than lexically; they carry no
//! `may`/`choose` morpheme and the stemming rule cannot see them. Treating those defs as offenders
//! would be a false positive, and quietly widening a stem to make them disappear is precisely the
//! failure this batch exists to remove. Each is therefore a row in
//! [`RECORDED_STRUCTURAL_EVIDENCE`] with a CR citation and a written reason, and
//! [`t_every_structural_evidence_row_is_live`] fails if a row stops matching anything.
//!
//! Two false-positive classes the brief warned about were **measured** rather than assumed:
//!
//! * *reminder text* — parenthesised spans are stripped before matching (CR 207.2: reminder text
//!   restates existing rules, it does not grant a new choice). Measured effect: this is
//!   load-bearing, see [`t_reminder_text_is_stripped`].
//! * *"may not"* — **zero occurrences** across all 1,803 defs' oracle text. The suppression the
//!   brief anticipated is not needed, and saying so with the count beats adding a rule that
//!   guards nothing. Pinned by [`t_may_not_is_measured_absent`] so the day one is authored, the
//!   claim fails instead of rotting.
//!
//! One further suppression is inherited rather than earned, and it is disclosed rather than
//! implied: the `PROSE_FIELDS` denylist on the DSL walk (a card's `name`/`oracle_text` spelling a
//! variant exactly) is **UNDISCRIMINATED** in this file's revert matrix (row V4) — removing it
//! changes nothing, because no prose string in today's corpus matches any `may*`/`cho*`/`up to`
//! identifier. It is kept because it is correct and free, not because it was measured to be
//! load-bearing here; `decision_site_walk`'s own `T3` is what proves the mechanism.
//!
//! ## What this gate is NOT
//!
//! It **cannot stop the growth; it makes it recorded** — the same bound `decision_gate.rs`'s `T4`
//! states about itself, for the same reason. Fixing an offender def is card-authoring work (and
//! for the `may` channel, mostly *engine* work: a costless "you may" on a trigger has no DSL
//! representation at all — audit §5 DP-12). The two legal exits are in the failure message.
//!
//! Its recall bound is also stated rather than implied. The closure reaches the three markers'
//! families and nothing else, so these attested optionality idioms are **outside** it and are
//! **not** measured by this gate: `unless` (54 occurrences), `any number of` (24), `rather than`
//! (17), `instead of` (5). Widening to them is a later batch's call; pretending they are covered
//! is the `OOS-DP7-11` class (a claim wearing a gate's authority).

use crate::decision_site_walk::{is_effectively_complete, PROSE_FIELDS};
use mtg_engine::{all_cards, CardDefinition};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

// ── Channels ──────────────────────────────────────────────────────────────────

/// One decision channel: an oracle-side marker and the DSL-side stem that expresses it.
///
/// The pairing is the point — a `choose`-shaped DSL construct does not discharge a printed
/// *"you may"*, and vice versa. Collapsing the channels into one "does this def carry any
/// decision at all" test is what let Smuggler's Copter pass: its incidental
/// `Effect::DiscardCards` is a decision site, just not the one its oracle text grants.
pub struct Channel {
    /// Stable id used in [`BASELINE`] rows and failure messages.
    pub id: &'static str,
    /// The marker the brief names. A single word is used as a **stem** (first three characters,
    /// [`STEM_LEN`]); a marker containing a space is matched as a whole phrase.
    pub marker: &'static str,
    /// CR anchor for "this printed wording grants the player a choice".
    pub cr: &'static str,
    /// What the channel means, for the failure message.
    pub what: &'static str,
}

pub const CHANNELS: &[Channel] = &[
    Channel {
        id: "may",
        marker: "may",
        // CR 601.2b covers optional additional costs on a cast; the general principle for a
        // resolution-time "you may" on a triggered/activated ability's effect is CR 608.2 (the
        // controller of the ability follows its instructions, and an instruction the card makes
        // optional is the controller's to decline).
        cr: "608.2 / 601.2b",
        what: "printed \"you may …\": the controller may decline the action entirely",
    },
    Channel {
        id: "choose",
        marker: "choose",
        // CR 700.2 (modal spells/abilities: the controller chooses a mode) and CR 609.4 (a choice
        // an effect instructs a player to make is made on resolution).
        cr: "700.2 / 609.4",
        what: "printed \"choose …\": the player picks among the printed options",
    },
    Channel {
        id: "up_to",
        marker: "up to",
        // CR 601.2c: "up to N targets" lets the caster choose how many, including zero.
        cr: "601.2c",
        what: "printed \"up to N …\": the player chooses how many, including zero",
    },
];

/// How many leading characters of a single-word marker form its stem. Three is the value that
/// makes `choose` reach `chosen` (which shares only `cho`), and it is stated here rather than
/// buried so the closure it produces can be re-derived.
const STEM_LEN: usize = 3;

/// Words admitted by the stem rule that are **not** decision vocabulary, each with its reason.
///
/// This is the only place the oracle axis is narrowed, and it exists because the stem rule is
/// deliberately mechanical: `may` at three characters also admits `mayhem`. Exactly one entry,
/// measured — and [`t_every_lexical_exclusion_is_live`] fails if it stops occurring, so a dead
/// exclusion cannot sit here masking a future real word.
const LEXICAL_EXCLUSIONS: &[(&str, &str)] = &[(
    "mayhem",
    "A card-name / flavour word ('Mayhem Devil'), not the modal 'may'. Admitted only because the \
     stem rule takes the first three characters of 'may'. 1 occurrence corpus-wide.",
)];

/// DSL constructs that express optionality **structurally**, carrying no `may`/`choose`/`up to`
/// morpheme for [`dsl_elements_for_stem`] to find. Each row is `(element, channel id, CR, reason)`.
///
/// These are **suppressions**, and every one is written down with its justification rather than
/// folded into a stem — the brief's standing instruction, and the `PROSE_FIELDS`/`T3` precedent
/// from PB-DP10. Measured effect on the `may` channel's effectively-`Complete` offender
/// population: 90 → 80 for the three `…Unless…` variants, and 80 → 72 once `unless_condition` is
/// counted, i.e. these rows suppress 18 defs that really do encode the choice.
pub const RECORDED_STRUCTURAL_EVIDENCE: &[(&str, &str, &str, &str)] = &[
    (
        "EntersTappedUnlessPayLife",
        "may",
        "614.1 / 118.12",
        "The shock-land replacement: 'As this land enters, you may pay 2 life' IS this variant. \
         The choice is encoded as an unless-payment, so the printed 'may' is honoured even though \
         the variant name carries no 'may' morpheme. 10 effectively-Complete defs.",
    ),
    (
        "CounterUnlessPays",
        "may",
        "118.12",
        "'Counter target spell unless its controller pays {N}' — CR 118.12's pay-or-not is a real \
         player choice and the engine serves it (decision_gate.rs row `counter_unless_pays`).",
    ),
    (
        "CantAttackYouUnlessPay",
        "may",
        "118.12 / 506.3",
        "A propaganda-style attack tax: the attacking player may pay or may not attack. Same \
         unless-payment shape as the two rows above.",
    ),
    (
        "unless_condition",
        "may",
        "118.12 / 614.1",
        "A non-null `unless_condition` on a replacement def is the DSL's general 'unless you …' \
         escape, and it is a field rather than a variant, so the identifier walk sees the key but \
         the stem rule cannot classify it. Counted only when non-null (a `None` encodes no \
         choice). 8 further effectively-Complete defs beyond the three variants above.",
    ),
    (
        "optional",
        "may",
        "608.2",
        "A boolean `optional: true` field is the DSL's explicit 'the controller may decline' flag. \
         Counted only when literally `true`: `optional: false` is the OPPOSITE claim, and treating \
         it as evidence would make the gate green on the exact shape it hunts.",
    ),
    (
        "modes",
        "choose",
        "700.2",
        "A non-null `modes` list IS 'choose one —': CR 700.2 makes mode selection the controller's \
         choice, and the DSL spells it structurally (a list of modes plus min/max) rather than \
         with a `Choose*` variant. Counted only when non-null.",
    ),
];

// ── The two derivations ───────────────────────────────────────────────────────

/// **Every** printed-text field this def carries, harvested STRUCTURALLY: every string value
/// under a key named `oracle_text`, at any depth in the serialized tree.
///
/// Found by the batch's inverse-method census (dispatch hygiene 6), and the first draft of this
/// file got it wrong: it read `def.oracle_text` alone. `CardFace` has its **own** `oracle_text`
/// (`crates/card-types/src/cards/card_definition.rs:30-44`), and a `CardDefinition` can carry two
/// of them — `back_face` (:77) and `adventure_face` (:114). A front-face-only read is blind to
/// every transformed face and every Adventure half, which is the same shape of hole as
/// `OOS-CARDS2-7` itself: a scan that measures the field its author remembered rather than the
/// fields the type actually has.
///
/// Harvesting by KEY NAME at arbitrary depth rather than by naming the three fields is what makes
/// a future face type (a third `CardFace` slot, a melded face) join automatically instead of
/// silently not joining — the PB-DX42a structural-walk lesson applied to prose.
fn printed_texts(def: &CardDefinition) -> Vec<String> {
    fn walk(v: &Value, out: &mut Vec<String>) {
        match v {
            Value::Object(m) => {
                for (k, child) in m {
                    if k == "oracle_text" {
                        if let Some(s) = child.as_str() {
                            out.push(s.to_string());
                        }
                    }
                    walk(child, out);
                }
            }
            Value::Array(a) => {
                for i in a {
                    walk(i, out);
                }
            }
            _ => {}
        }
    }
    let mut out = Vec::new();
    walk(
        &serde_json::to_value(def).expect("CardDefinition serializes"),
        &mut out,
    );
    out
}

/// [`oracle_words`] over every printed-text field on the def, front face and all other faces.
fn all_oracle_words(def: &CardDefinition) -> Vec<String> {
    printed_texts(def)
        .iter()
        .flat_map(|t| oracle_words(t))
        .collect()
}

/// Strip parenthesised reminder text (CR 207.2), lower-case, and tokenize.
fn oracle_words(text: &str) -> Vec<String> {
    let mut out = String::with_capacity(text.len());
    let mut depth = 0usize;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(ch.to_ascii_lowercase()),
            _ => {}
        }
    }
    out.split(|c: char| !(c.is_ascii_alphabetic() || c == '-' || c == '\''))
        .filter(|w| !w.is_empty())
        .map(|w| w.to_string())
        .collect()
}

/// The corpus's own inflected family for a single-word marker: every whole word in any def's
/// oracle text beginning with the marker's [`STEM_LEN`]-character stem, minus
/// [`LEXICAL_EXCLUSIONS`].
///
/// Derived from `all_cards()` on every call rather than transcribed — that is what makes it a
/// derivation and not a list.
pub fn decision_word_closure(defs: &[CardDefinition], marker: &str) -> BTreeSet<String> {
    let stem: String = marker.chars().take(STEM_LEN).collect();
    let excluded: BTreeSet<&str> = LEXICAL_EXCLUSIONS.iter().map(|(w, _)| *w).collect();
    let mut out = BTreeSet::new();
    for d in defs {
        for w in all_oracle_words(d) {
            if w.starts_with(&stem) && !excluded.contains(w.as_str()) {
                out.insert(w);
            }
        }
    }
    out
}

/// Split a Rust identifier (`ChooseColor`, `unless_condition`) into lower-cased word tokens.
fn identifier_tokens(ident: &str) -> Vec<String> {
    let mut spaced = String::with_capacity(ident.len() * 2);
    let mut prev_lower_or_digit = false;
    for ch in ident.chars() {
        if ch == '_' {
            spaced.push(' ');
            prev_lower_or_digit = false;
            continue;
        }
        if ch.is_ascii_uppercase() && prev_lower_or_digit {
            spaced.push(' ');
        }
        prev_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        spaced.push(ch.to_ascii_lowercase());
    }
    spaced
        .split_whitespace()
        .filter(|t| t.chars().all(|c| c.is_ascii_alphabetic()))
        .map(|t| t.to_string())
        .collect()
}

/// Every object key and bare variant string reachable in the serialized corpus.
///
/// Bare strings are collected only when their parent key is **not** a `PROSE_FIELDS` entry — the
/// `decision_site_walk::T3` suppression, reused rather than re-derived: a card's `oracle_text` or
/// `name` spelling a variant exactly is not a DSL element.
pub fn dsl_surface(defs: &[CardDefinition]) -> BTreeSet<String> {
    fn walk(v: &Value, parent: Option<&str>, out: &mut BTreeSet<String>) {
        match v {
            Value::Object(m) => {
                for (k, child) in m {
                    out.insert(k.clone());
                    walk(child, Some(k.as_str()), out);
                }
            }
            Value::Array(a) => {
                for i in a {
                    walk(i, parent, out);
                }
            }
            Value::String(s) => {
                let suppressed = parent.map(|k| PROSE_FIELDS.contains(&k)).unwrap_or(false);
                if !suppressed
                    && !s.is_empty()
                    && s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && s.chars().all(|c| c.is_ascii_alphanumeric())
                {
                    out.insert(s.clone());
                }
            }
            _ => {}
        }
    }
    let mut out = BTreeSet::new();
    for d in defs {
        walk(
            &serde_json::to_value(d).expect("CardDefinition serializes"),
            None,
            &mut out,
        );
    }
    out
}

/// The DSL elements that can express this channel's choice: identifiers carrying a token that
/// begins with the same stem the oracle axis uses (or, for a phrase marker, whose joined tokens
/// contain the phrase).
pub fn dsl_elements_for_stem(surface: &BTreeSet<String>, marker: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for e in surface {
        let toks = identifier_tokens(e);
        let hit = if marker.contains(' ') {
            toks.join(" ").contains(marker)
        } else {
            let stem: String = marker.chars().take(STEM_LEN).collect();
            toks.iter().any(|t| t.starts_with(&stem))
        };
        if hit {
            out.insert(e.clone());
        }
    }
    out
}

// ── Per-def evaluation ────────────────────────────────────────────────────────

/// A def's serialized facts, computed once per def rather than once per channel.
struct DefFacts {
    /// Every object key and (prose-suppressed) bare variant string in this def's own tree.
    elements: BTreeSet<String>,
    /// Keys whose value is literally `true` somewhere in this def.
    truthy_keys: BTreeSet<String>,
    /// Keys whose value is non-null somewhere in this def.
    nonnull_keys: BTreeSet<String>,
}

fn def_facts(def: &CardDefinition) -> DefFacts {
    fn walk(
        v: &Value,
        parent: Option<&str>,
        elements: &mut BTreeSet<String>,
        truthy: &mut BTreeSet<String>,
        nonnull: &mut BTreeSet<String>,
    ) {
        match v {
            Value::Object(m) => {
                for (k, child) in m {
                    elements.insert(k.clone());
                    if !child.is_null() {
                        nonnull.insert(k.clone());
                    }
                    if child.as_bool() == Some(true) {
                        truthy.insert(k.clone());
                    }
                    walk(child, Some(k.as_str()), elements, truthy, nonnull);
                }
            }
            Value::Array(a) => {
                for i in a {
                    walk(i, parent, elements, truthy, nonnull);
                }
            }
            Value::String(s) => {
                let suppressed = parent.map(|k| PROSE_FIELDS.contains(&k)).unwrap_or(false);
                if !suppressed
                    && !s.is_empty()
                    && s.chars().next().is_some_and(|c| c.is_ascii_uppercase())
                    && s.chars().all(|c| c.is_ascii_alphanumeric())
                {
                    elements.insert(s.clone());
                }
            }
            _ => {}
        }
    }
    let mut f = DefFacts {
        elements: BTreeSet::new(),
        truthy_keys: BTreeSet::new(),
        nonnull_keys: BTreeSet::new(),
    };
    walk(
        &serde_json::to_value(def).expect("CardDefinition serializes"),
        None,
        &mut f.elements,
        &mut f.truthy_keys,
        &mut f.nonnull_keys,
    );
    f
}

/// Does this def's PRINTED text grant a choice of `channel`'s kind?
fn oracle_grants(def: &CardDefinition, channel: &Channel, closure: &BTreeSet<String>) -> bool {
    let words = all_oracle_words(def);
    if channel.marker.contains(' ') {
        words.join(" ").contains(channel.marker)
    } else {
        words.iter().any(|w| closure.contains(w))
    }
}

/// Does this def's DSL carry anything able to EXPRESS a choice of `channel`'s kind?
fn dsl_expresses(facts: &DefFacts, channel: &Channel, elements: &BTreeSet<String>) -> bool {
    if !facts.elements.is_disjoint(elements) {
        return true;
    }
    for (element, chan, _, _) in RECORDED_STRUCTURAL_EVIDENCE {
        if *chan != channel.id {
            continue;
        }
        // `optional` is evidence only when literally true; `modes` / `unless_condition` only when
        // non-null; a named variant is evidence by presence.
        let satisfied = match *element {
            "optional" => facts.truthy_keys.contains(*element),
            "modes" | "unless_condition" => facts.nonnull_keys.contains(*element),
            _ => facts.elements.contains(*element),
        };
        if satisfied {
            return true;
        }
    }
    false
}

/// The channels on which `def` prints a choice its DSL cannot express. Empty = clean.
pub fn dropped_channels(
    def: &CardDefinition,
    closures: &ChannelVocabulary,
    elements: &ChannelVocabulary,
) -> BTreeSet<&'static str> {
    let facts = def_facts(def);
    CHANNELS
        .iter()
        .filter(|c| {
            oracle_grants(def, c, &closures[c.id]) && !dsl_expresses(&facts, c, &elements[c.id])
        })
        .map(|c| c.id)
        .collect()
}

/// Per-channel derived vocabulary: the oracle-side word closure and the DSL-side element set.
type ChannelVocabulary = BTreeMap<&'static str, BTreeSet<String>>;

fn closures_and_elements(defs: &[CardDefinition]) -> (ChannelVocabulary, ChannelVocabulary) {
    let surface = dsl_surface(defs);
    let mut closures = BTreeMap::new();
    let mut elements = BTreeMap::new();
    for c in CHANNELS {
        closures.insert(c.id, decision_word_closure(defs, c.marker));
        elements.insert(c.id, dsl_elements_for_stem(&surface, c.marker));
    }
    (closures, elements)
}

// ── The recorded baseline ─────────────────────────────────────────────────────

/// `(def name exactly as `all_cards()` reports it, the exact sorted set of channel ids on which
/// its printed choice is unexpressed, a post-freeze reason)`. `None` = frozen at the 2026-08-12
/// PB-DX8 measurement; `Some(text)` = a later, deliberate addition, which must carry its own
/// reason ([`t_every_baseline_entry_is_live_and_necessary`]).
///
/// **An entry asserts exactly one thing — that this def's printed text grants a choice on these
/// channels and its DSL carries nothing able to express it. It asserts NOTHING about whether the
/// def is otherwise oracle-correct, and NOTHING about how bad the resulting behaviour is.**
///
/// **This roster was populated MECHANICALLY from the measurement below and was NOT adjudicated
/// against oracle text def-by-def.** That sentence is here because PB-DP10's own `BASELINE`
/// shipped implying the opposite and its review had to correct it (see
/// `decision_gate.rs:356-382`); the correction is cheaper to write once than to discover twice.
/// Spot-reading during the freeze found both shapes present, and they are genuinely different:
///
/// * *the choice is dropped* — `Aura Shards` ("you **may** destroy target artifact or
///   enchantment") and `Eternal Witness` ("you **may** return target card") author the effect
///   unconditionally, so the controller is forced to act. This is Smuggler's Copter's class.
/// * *the choice is elsewhere* — `Force of Will` ("you **may** pay 1 life and exile a blue card
///   … rather than pay this spell's mana cost") expresses its optionality as an alternative cost,
///   which is a real DSL construct this gate's stemming rule cannot see. A future batch may
///   convert entries of this second shape into [`RECORDED_STRUCTURAL_EVIDENCE`] rows once the
///   construct is identified by name; doing it now, unmeasured, would be needle tuning.
///
/// Read this list as *"the printed choice is not expressed by a construct this gate recognises"*,
/// which is exactly what it measures — not as a defect ledger.
pub const BASELINE: &[(&str, &[&str], Option<&str>)] = &[
    ("Ancient Greenwarden", &["may"], None),
    ("Aura Shards", &["may"], None),
    ("Avenger of Zendikar", &["may"], None),
    ("Awakening Zone", &["may"], None),
    ("Azusa, Lost but Seeking", &["may"], None),
    ("Beastmaster Ascension", &["may"], None),
    ("Bladewing the Risen", &["may"], None),
    ("Blighted Woodland", &["up_to"], None),
    ("Bloodline Necromancer", &["may"], None),
    ("Borne Upon a Wind", &["may"], None),
    ("Boseiju, Who Endures", &["may"], None),
    ("Broken Bond", &["may"], None),
    ("Brokkos, Apex of Forever", &["may"], None),
    ("Burgeoning", &["may"], None),
    ("Case of the Locked Hothouse", &["may"], None),
    ("Chulane, Teller of Tales", &["may"], None),
    ("Coastal Piracy", &["may"], None),
    ("Combat Celebrant", &["may"], None),
    ("Complete the Circuit", &["may", "choose"], None),
    ("Courser of Kruphix", &["may"], None),
    ("Crucible of Worlds", &["may"], None),
    ("Cultivate", &["up_to"], None),
    ("Deadly Rollick", &["may"], None),
    ("Deflecting Swat", &["may", "choose"], None),
    ("Delver of Secrets", &["may"], None),
    ("Dryad of the Ilysian Grove", &["may"], None),
    ("Elven Chorus", &["may"], None),
    ("Elvish Harbinger", &["may"], None),
    ("Eternal Witness", &["may"], None),
    ("Exploration", &["may"], None),
    ("Explore", &["may"], None),
    ("Explosive Vegetation", &["up_to"], None),
    ("Farhaven Elf", &["may"], None),
    ("Fierce Empath", &["may"], None),
    ("Fierce Guardianship", &["may"], None),
    ("Flawless Maneuver", &["may"], None),
    ("Force of Negation", &["may"], None),
    ("Force of Vigor", &["may"], None),
    ("Force of Will", &["may"], None),
    ("Forerunner of the Legion", &["may"], None),
    ("Future Sight", &["may"], None),
    ("Goblin Matron", &["may"], None),
    ("Growth Spiral", &["may"], None),
    ("Harrow", &["up_to"], None),
    ("Higure, the Still Wind", &["may"], None),
    ("Icetill Explorer", &["may"], None),
    ("Khalni Heart Expedition", &["may", "up_to"], None),
    ("Kodama's Reach", &["up_to"], None),
    ("Land Tax", &["may", "up_to"], None),
    ("Misdirection", &["may"], None),
    ("Mistblade Shinobi", &["may"], None),
    ("Ninja of the Deep Hours", &["may"], None),
    ("Oathsworn Vampire", &["may"], None),
    ("Oracle of Mul Daya", &["may"], None),
    ("Pawn of Ulamog", &["may"], None),
    ("Perennial Behemoth", &["may"], None),
    ("Quest for the Goblin Lord", &["may"], None),
    ("Radha, Heart of Keld", &["may"], None),
    ("Ramunap Excavator", &["may"], None),
    ("Reclamation Sage", &["may"], None),
    ("Reconnaissance Mission", &["may"], None),
    ("Recruiter of the Guard", &["may"], None),
    ("Roiling Regrowth", &["up_to"], None),
    ("Sakura-Tribe Scout", &["may"], None),
    ("Skyshroud Claim", &["up_to"], None),
    ("Solemn Simulacrum", &["may"], None),
    ("Spellseeker", &["may"], None),
    ("Springbloom Druid", &["up_to"], None),
    ("Squee, Dubious Monarch", &["may"], None),
    ("Sword of Light and Shadow", &["may"], None),
    ("Teferi, Time Raveler", &["may"], None),
    ("Teneb, the Harvester", &["may"], None),
    ("Thundermane Dragon", &["may"], None),
    ("Timeline Culler", &["may"], None),
    ("Urban Evolution", &["may"], None),
    ("Windbrisk Heights", &["may"], None),
    ("Wirewood Herald", &["may"], None),
    ("World Shaper", &["may"], None),
    ("Xenagos, the Reveler", &["may"], None),
    ("Yeva, Nature's Herald", &["may"], None),
];

/// Entries frozen by the 2026-08-12 PB-DX8 measurement. Every entry added AFTER that freeze must
/// carry `Some(reason)`; this ceiling is what enforces the promise the failure message makes.
/// Deliberately does not grow — shrinking is fine (a def repaired or demoted simply leaves).
const FROZEN_2026_08_12: usize = 80;

/// Exact pin on the union of effectively-`Complete` defs carrying at least one dropped channel.
/// A per-def check alone cannot close the hole where a NEW def slots into a channel that already
/// has entries — `decision_gate.rs`'s `T6` exists for the same reason.
const COMPLETE_DROPPED_UNION: usize = 80;

/// Non-vacuity floors. Each is a `>=` denominator guard, not a target: if the walk breaks, every
/// assertion in this file passes by finding nothing (the absence-shaped vacuity PB-DX6's two
/// empty-pinned rosters warned about).
const MIN_CORPUS: usize = 1000;
/// The `may` channel's oracle-positive population across the whole corpus, Complete or not.
const MIN_MAY_ORACLE_HITS: usize = 250;
/// Distinct DSL elements the `cho` stem reaches. Measured 20; floored well below so ordinary
/// authoring does not redden it, but a stemming bug that returns nothing does.
const MIN_CHOOSE_DSL_ELEMENTS: usize = 10;

// ── The gate ──────────────────────────────────────────────────────────────────

fn baseline_map() -> BTreeMap<&'static str, BTreeSet<&'static str>> {
    BASELINE
        .iter()
        .map(|(name, chans, _)| (*name, chans.iter().copied().collect()))
        .collect()
}

/// The gate's offender-detection logic, extracted so the gate and its own non-vacuity probe drive
/// the IDENTICAL code path. PB-DP10's review finding #3 is the precedent: its probe re-checked
/// predicates other tests already covered and never executed this loop at all, so the
/// subset/superset mismatch arm — half the ratchet's design rationale — had no coverage anywhere.
fn offenders(
    defs: &[CardDefinition],
    baseline: &BTreeMap<&str, BTreeSet<&'static str>>,
    closures: &ChannelVocabulary,
    elements: &ChannelVocabulary,
) -> Vec<String> {
    let mut out = Vec::new();
    for def in defs {
        if !is_effectively_complete(def) {
            continue;
        }
        let dropped = dropped_channels(def, closures, elements);
        if dropped.is_empty() {
            continue;
        }
        match baseline.get(def.name.as_str()) {
            None => {
                let detail: Vec<String> = dropped
                    .iter()
                    .map(|id| {
                        let c = CHANNELS.iter().find(|c| c.id == *id).expect("channel id");
                        format!("{id} (CR {}, {})", c.cr, c.what)
                    })
                    .collect();
                out.push(format!(
                    "{} is NOT in BASELINE but drops {dropped:?}. {}",
                    def.name,
                    detail.join("; ")
                ));
            }
            Some(recorded) if recorded != &dropped => {
                out.push(format!(
                    "{} is in BASELINE with channels {recorded:?} but the corpus now shows \
                     {dropped:?} (superset = the def lost a choice since the freeze, or gained \
                     printed text; subset = tighten the entry)",
                    def.name
                ));
            }
            _ => {}
        }
    }
    out
}

/// The gate's failure message. Extracted so [`t_failure_message_names_the_bound`] can assert
/// against it directly, rather than a module doc citing a test that does not exist — the exact
/// gap PB-DP10's review finding #5 found in `decision_gate.rs`.
fn gate_message(offenders: &[String]) -> String {
    format!(
        "These effectively-Complete card defs PRINT a choice their DSL cannot express, so the \
         engine performs an action the printed card lets the player decline (OOS-DP10-9; \
         docs/audits/decision-point-audit.md §3.1). This is strictly worse than the class \
         decision_gate.rs records: a recorded auto-choice is at least a legal outcome, a dropped \
         \"may\" is not.\n\n\
         THIS GATE CANNOT STOP THE GROWTH; IT MAKES IT RECORDED. Two legal exits, and only two:\n\
         1. Mark the def non-Complete with a note naming the dropped clause — \
            `completeness: Completeness::known_wrong(\"printed 'you may X' authored as an \
            unconditional Sequence\")`.\n\
         2. Add a BASELINE entry in this file with the def's exact channel set AND a written \
            reason.\n\n\
         Implementing the choice properly is NOT an exit for this batch: a costless \"you may\" on \
         a trigger has no DSL representation at all and needs the owning engine PB (audit §5, \
         DP-12), not a card-def edit. If instead the DSL DOES express this choice through a \
         construct the stemming rule cannot see, add a RECORDED_STRUCTURAL_EVIDENCE row with its \
         CR citation and reason — never widen a stem silently.\n\nOffenders:\n{}",
        offenders.join("\n")
    )
}

#[test]
/// **The gate.** See [`offenders`] and [`gate_message`].
fn no_complete_def_drops_a_printed_choice_unrecorded() {
    let defs = all_cards();
    let (closures, elements) = closures_and_elements(&defs);
    let found = offenders(&defs, &baseline_map(), &closures, &elements);
    assert!(found.is_empty(), "{}", gate_message(&found));
}

#[test]
/// The failure message must carry the bound and both exits, so a reader cannot mistake this gate
/// for a closure of the class it records.
fn t_failure_message_names_the_bound() {
    let msg = gate_message(&["Fake Offender is NOT in BASELINE but drops {\"may\"}".to_string()]);
    for phrase in [
        "CANNOT STOP THE GROWTH",
        "Mark the def non-Complete",
        "Add a BASELINE entry",
        "is NOT an exit for this batch",
        "RECORDED_STRUCTURAL_EVIDENCE",
    ] {
        assert!(
            msg.contains(phrase),
            "the gate's failure message must contain {phrase:?}; got:\n{msg}"
        );
    }
}

#[test]
/// The gate logic is not vacuously green. Drives the SAME [`offenders`] function against a
/// synthetic three-def corpus, never touching `all_cards()`, and exercises all three outcomes:
/// unbaselined offender, baselined-but-mismatched offender, and non-`Complete` non-offender.
fn t_gate_logic_reddens_on_a_new_unbaselined_dropper() {
    fn may_def(name: &str) -> CardDefinition {
        CardDefinition {
            name: name.to_string(),
            // Printed "you may", authored as nothing at all — Smuggler's Copter's shape.
            oracle_text: "When this creature enters, you may draw a card.".to_string(),
            ..Default::default()
        }
    }
    let mut unbaselined = may_def("PB-DX8 Synthetic Dropper (unbaselined)");
    unbaselined.completeness = mtg_engine::cards::Completeness::Complete;
    let mut mismatched = may_def("PB-DX8 Synthetic Dropper (mismatched baseline)");
    mismatched.completeness = mtg_engine::cards::Completeness::Complete;
    let mut non_complete = may_def("PB-DX8 Synthetic Non-Offender (not Complete)");
    non_complete.completeness = mtg_engine::cards::Completeness::partial("probe, not real");

    // The mismatch arm: recorded channel set is a strict SUPERSET of what the def actually drops.
    let baseline: BTreeMap<&str, BTreeSet<&'static str>> = [(
        mismatched.name.as_str(),
        ["may", "up_to"].into_iter().collect(),
    )]
    .into_iter()
    .collect();

    // The closures/elements must come from the REAL corpus: the synthetic defs carry no DSL, and
    // deriving the vocabulary from them alone would make every channel empty and the probe vacuous.
    let real = all_cards();
    let (closures, elements) = closures_and_elements(&real);
    let corpus = [
        unbaselined.clone(),
        mismatched.clone(),
        non_complete.clone(),
    ];
    let found = offenders(&corpus, &baseline, &closures, &elements);

    assert!(
        found
            .iter()
            .any(|o| o.contains(unbaselined.name.as_str()) && o.contains("is NOT in BASELINE")),
        "(a) an unbaselined Complete def printing \"you may\" with no DSL evidence must be an \
         offender: {found:?}"
    );
    assert!(
        found
            .iter()
            .any(|o| o.contains(mismatched.name.as_str()) && o.contains("tighten the entry")),
        "(b) the recorded-superset mismatch arm must fire: {found:?}"
    );
    assert!(
        !found.iter().any(|o| o.contains(non_complete.name.as_str())),
        "(c) a non-Complete def must never be an offender, even with the identical text: {found:?}"
    );
    assert_eq!(
        found.len(),
        2,
        "exactly two of the three are offenders: {found:?}"
    );
}

#[test]
/// Every `BASELINE` entry names a real, still-`Complete` def whose CURRENT dropped-channel set
/// equals the recorded one exactly, and every post-freeze entry carries a written reason.
fn t_every_baseline_entry_is_live_and_necessary() {
    let defs = all_cards();
    let (closures, elements) = closures_and_elements(&defs);
    let by_name: BTreeMap<&str, &CardDefinition> =
        defs.iter().map(|d| (d.name.as_str(), d)).collect();

    for (name, chans, reason) in BASELINE {
        let def = by_name
            .get(*name)
            .unwrap_or_else(|| panic!("BASELINE names {name:?}, which is not in all_cards()"));
        assert!(
            is_effectively_complete(def),
            "BASELINE entry {name:?} is no longer Complete — it passes on the marker now; remove \
             the redundant entry"
        );
        let recorded: BTreeSet<&'static str> = chans.iter().copied().collect();
        let actual = dropped_channels(def, &closures, &elements);
        assert_eq!(
            actual, recorded,
            "BASELINE entry {name:?} recorded channels {recorded:?} but the corpus now shows \
             {actual:?} — a superset means this def dropped a further choice since the freeze \
             (investigate, don't just widen the entry); a subset means the entry should be \
             tightened"
        );
        if let Some(text) = reason {
            assert!(
                text.len() >= 30,
                "BASELINE entry {name:?}'s post-freeze reason is too short ({} chars); a recorded \
                 acknowledgement needs a real sentence, not a stub",
                text.len()
            );
        }
    }

    let unexplained = BASELINE.iter().filter(|(_, _, r)| r.is_none()).count();
    assert!(
        unexplained <= FROZEN_2026_08_12,
        "{unexplained} BASELINE entries carry no written reason, but only the \
         {FROZEN_2026_08_12} entries of the 2026-08-12 PB-DX8 freeze are allowed to. Every entry \
         added after that freeze is a deliberate act and must carry `Some(reason)` — that is what \
         the gate's failure message promises an author, and this is where the promise is kept."
    );
}

#[test]
/// The union ratchet. A per-def check cannot close the hole where a NEW def slots into a channel
/// that already has entries; this pins the aggregate. One cause, two red tests: if this fails,
/// read the gate's message first — it usually names the def.
fn t_complete_dropped_union_is_ratcheted() {
    let defs = all_cards();
    assert!(
        defs.len() >= MIN_CORPUS,
        "the corpus shrank to {} (< {MIN_CORPUS}) — denominator guard",
        defs.len()
    );
    let (closures, elements) = closures_and_elements(&defs);
    let union: BTreeSet<String> = defs
        .iter()
        .filter(|d| is_effectively_complete(d))
        .filter(|d| !dropped_channels(d, &closures, &elements).is_empty())
        .map(|d| d.name.clone())
        .collect();

    assert_eq!(
        union.len(),
        COMPLETE_DROPPED_UNION,
        "the effectively-Complete dropped-choice union moved to {} from the pinned {}. GREW: a \
         new Complete def prints a choice its DSL cannot express — see the gate's own failure \
         message first (it usually names the def), then either demote it or add a BASELINE entry \
         and raise COMPLETE_DROPPED_UNION in the SAME commit. SHRANK: good — a def was repaired, \
         demoted, or a new RECORDED_STRUCTURAL_EVIDENCE row now covers it; lower the pin (and \
         prune the stale BASELINE entries the liveness test will name) so the ratchet keeps the \
         gain.",
        union.len(),
        COMPLETE_DROPPED_UNION
    );
}

// ── Derivation pins and non-vacuity ───────────────────────────────────────────

#[test]
/// The oracle-side morphological closure, pinned. A new inflection joining it is legitimate — the
/// closure is derived, not listed — but it must be VISIBLE, because it widens the gate's reach
/// over the whole corpus in one step.
fn t_decision_word_closure_is_pinned() {
    let defs = all_cards();
    let may: Vec<String> = decision_word_closure(&defs, "may").into_iter().collect();
    let cho: Vec<String> = decision_word_closure(&defs, "choose").into_iter().collect();
    assert_eq!(
        may,
        vec!["may".to_string()],
        "the `may` stem's corpus closure moved. If a real new inflection appeared, update this pin \
         with a dated note; if it is another `may*` word that is not the modal (the `mayhem` \
         shape), add a LEXICAL_EXCLUSIONS row with its reason instead of widening the pin"
    );
    assert_eq!(
        cho,
        vec![
            "choice".to_string(),
            "choose".to_string(),
            "chooses".to_string(),
            "chosen".to_string()
        ],
        "the `cho` stem's corpus closure moved — same two exits as the `may` pin above"
    );
}

#[test]
/// The DSL-side derivation must actually find elements. A stemming or walk bug returns an empty
/// set, and an empty DSL side makes EVERY oracle-positive def an offender — a failure that would
/// be loud, or, if the oracle side broke in the same commit, silent.
fn t_dsl_element_derivation_is_not_vacuous() {
    let defs = all_cards();
    let surface = dsl_surface(&defs);
    assert!(
        surface.len() > 500,
        "the DSL surface walk found only {} elements; it is not reaching the corpus and every \
         channel's evidence set is vacuous",
        surface.len()
    );
    let may = dsl_elements_for_stem(&surface, "may");
    let cho = dsl_elements_for_stem(&surface, "choose");
    let up = dsl_elements_for_stem(&surface, "up to");
    assert!(
        may.contains("MayPayOrElse") && may.contains("MayPayThenEffect"),
        "the `may` stem must reach both MayPay* variants; got {may:?}"
    );
    assert!(
        cho.len() >= MIN_CHOOSE_DSL_ELEMENTS,
        "the `cho` stem reached only {} DSL elements (< {MIN_CHOOSE_DSL_ELEMENTS}); measured 20 at \
         the 2026-08-12 freeze, so this is a stemming or walk regression, not authoring drift",
        cho.len()
    );
    assert!(
        up.contains("UpToN"),
        "the `up to` phrase marker must reach UpToN; got {up:?}"
    );
}

#[test]
/// The oracle side must actually fire on the corpus. Absence-shaped vacuity: if the closure or the
/// reminder-stripper broke, the gate would pass by finding nothing.
fn t_oracle_side_is_not_vacuous() {
    let defs = all_cards();
    let closure = decision_word_closure(&defs, "may");
    let may = CHANNELS
        .iter()
        .find(|c| c.id == "may")
        .expect("may channel");
    let hits = defs
        .iter()
        .filter(|d| oracle_grants(d, may, &closure))
        .count();
    assert!(
        hits >= MIN_MAY_ORACLE_HITS,
        "only {hits} defs' oracle text matched the `may` channel (< {MIN_MAY_ORACLE_HITS}); \
         measured 285 at the freeze. The closure or the tokenizer is broken and the gate is \
         vacuous"
    );
}

#[test]
/// Reminder text is stripped (CR 207.2), and the stripping is load-bearing rather than decorative:
/// the same sentence inside and outside parentheses must give opposite answers.
fn t_reminder_text_is_stripped() {
    assert!(
        !oracle_words("Crew 1 (Tap any number of creatures ... you may do this)")
            .iter()
            .any(|w| w == "may"),
        "a `may` inside reminder text must not be seen — CR 207.2 reminder text restates existing \
         rules, it does not grant a new choice"
    );
    assert!(
        oracle_words("When this enters, you may draw a card.")
            .iter()
            .any(|w| w == "may"),
        "the identical word OUTSIDE parentheses must be seen — otherwise the stripper is eating \
         the whole text and every def is a false negative"
    );
}

#[test]
/// The brief anticipated a `may not` false-positive class. It is MEASURED ABSENT — zero
/// occurrences across the whole corpus — so no suppression is written for it. Pinned so the claim
/// fails on the day one is authored rather than rotting into a stale assertion.
fn t_may_not_is_measured_absent() {
    let offenders: Vec<String> = all_cards()
        .iter()
        .filter(|d| all_oracle_words(d).windows(2).any(|w| w == ["may", "not"]))
        .map(|d| d.name.clone())
        .collect();
    assert!(
        offenders.is_empty(),
        "the corpus now contains `may not`, which this file's module doc records as measured \
         absent and therefore needs no suppression. A printed \"may not\" is a PROHIBITION, not a \
         grant, so the `may` channel would now produce false positives on {offenders:?}. Add a \
         suppression with its reason and update the module doc's count."
    );
}

#[test]
/// Every [`RECORDED_STRUCTURAL_EVIDENCE`] row must still suppress something, name a real channel,
/// and carry a real reason. A dead suppression is not harmless: it sits in the file reading as
/// justification for a narrowing that no longer happens, and the next reader trusts it.
fn t_every_structural_evidence_row_is_live() {
    let defs = all_cards();
    let facts: Vec<DefFacts> = defs.iter().map(def_facts).collect();
    for (element, chan, cr, reason) in RECORDED_STRUCTURAL_EVIDENCE {
        assert!(
            CHANNELS.iter().any(|c| c.id == *chan),
            "structural evidence row {element:?} names channel {chan:?}, which is not a channel"
        );
        assert!(
            reason.len() >= 40 && !cr.is_empty(),
            "structural evidence row {element:?} needs a CR citation and a real reason"
        );
        let live = facts.iter().any(|f| match *element {
            "optional" => f.truthy_keys.contains(*element),
            "modes" | "unless_condition" => f.nonnull_keys.contains(*element),
            _ => f.elements.contains(*element),
        });
        assert!(
            live,
            "structural evidence row {element:?} no longer matches any def in the corpus — it is \
             suppressing nothing. Remove it, or find out why the construct vanished"
        );
    }
}

#[test]
/// `optional: false` is the OPPOSITE claim to `optional: true`, and counting the key's mere
/// presence as evidence would make the gate green on exactly the shape it hunts. This is the
/// `FieldCoverage::Full` mistake PB-DX7's review found — "the token appears" is not "the value
/// says yes".
///
/// **This probe drives the production [`dsl_expresses`] directly**, with synthetic [`DefFacts`],
/// rather than re-implementing the truthiness check locally. Its first draft did the latter and
/// was UNDISCRIMINATED under the matching revert row (V6): flipping `dsl_expresses` to count key
/// PRESENCE left it green, because it never executed that function. That is PB-DP10 review
/// finding #3 recurring verbatim — a probe that checks a predicate the gate does not use.
///
/// The revert row is discriminating for a second reason worth recording rather than assuming:
/// **measured, only 5 defs in the whole corpus carry an `optional` key at all and all 5 have it
/// `true`**, so the live corpus cannot distinguish the two readings. Nothing but this synthetic
/// probe stands between the gate and that regression.
fn t_optional_false_is_not_evidence() {
    let may = CHANNELS
        .iter()
        .find(|c| c.id == "may")
        .expect("may channel");
    let no_lexical: BTreeSet<String> = BTreeSet::new();

    let mut yes = DefFacts {
        elements: BTreeSet::new(),
        truthy_keys: BTreeSet::new(),
        nonnull_keys: BTreeSet::new(),
    };
    yes.elements.insert("optional".to_string());
    yes.truthy_keys.insert("optional".to_string());
    yes.nonnull_keys.insert("optional".to_string());
    assert!(
        dsl_expresses(&yes, may, &no_lexical),
        "`optional: true` MUST count as evidence that the printed choice is expressed"
    );

    // The key is present and non-null (serde writes `false`, not `null`) but its VALUE says no.
    let mut no = DefFacts {
        elements: BTreeSet::new(),
        truthy_keys: BTreeSet::new(),
        nonnull_keys: BTreeSet::new(),
    };
    no.elements.insert("optional".to_string());
    no.nonnull_keys.insert("optional".to_string());
    assert!(
        !dsl_expresses(&no, may, &no_lexical),
        "`optional: false` must NOT count as evidence — the key is there and the answer is no. A \
         presence check here would green-light exactly the dropped-choice shape this gate hunts"
    );
}

#[test]
/// Smuggler's Copter — `OOS-DP10-8`, the seed's own worked example — is in the MEASURED
/// population and is seen by the instrument for the RIGHT reason.
///
/// It is `known_wrong` (PB-DX4 demoted it), so it is correctly not an *offender*: the marker
/// already declares the defect. What this pins is that the scanner reaches it, classifies its
/// printed "you may draw a card" as a `may`-channel grant, and finds NO may-shaped DSL construct —
/// i.e. this gate detects the defect itself, where `decision_gate.rs` saw the card only through
/// the incidental `Effect::DiscardCards` inside the same unconditional `Sequence`.
fn t_smugglers_copter_is_in_the_measured_population() {
    let defs = all_cards();
    let (closures, elements) = closures_and_elements(&defs);
    let copter = defs
        .iter()
        .find(|d| d.name == "Smuggler's Copter")
        .expect("Smuggler's Copter must be in all_cards() — it is this seed's worked example");

    let may = CHANNELS
        .iter()
        .find(|c| c.id == "may")
        .expect("may channel");
    assert!(
        oracle_grants(copter, may, &closures["may"]),
        "Smuggler's Copter's printed \"you may draw a card\" must register on the `may` channel"
    );
    assert!(
        !dsl_expresses(&def_facts(copter), may, &elements["may"]),
        "Smuggler's Copter must carry NO may-shaped DSL construct — its may-clause is authored as \
         a bare Effect::Sequence(DrawCards, DiscardCards). If this now fails, the card was \
         repaired: delete this assertion and celebrate, do not weaken it"
    );
    assert_eq!(
        dropped_channels(copter, &closures, &elements),
        ["may"].into_iter().collect::<BTreeSet<_>>(),
        "the dropped-channel set for Smuggler's Copter is exactly {{may}}"
    );
    assert!(
        !is_effectively_complete(copter),
        "Smuggler's Copter is `known_wrong` (PB-DX4), so it is measured but not an offender — the \
         marker is the declaration this gate's exit 1 asks for. If it flips back to Complete, it \
         belongs in BASELINE"
    );
}

#[test]
/// The channel split is load-bearing. A single "does this def carry any decision at all" test
/// would pass Smuggler's Copter on its incidental `Effect::DiscardCards`; pairing each oracle
/// marker with its OWN DSL evidence is what makes the gate see the dropped clause. Pinned
/// directly, because collapsing the channels is the most tempting simplification a future reader
/// could make to this file.
fn t_channels_are_not_interchangeable() {
    let defs = all_cards();
    let (_closures, elements) = closures_and_elements(&defs);
    let copter = defs
        .iter()
        .find(|d| d.name == "Smuggler's Copter")
        .expect("Smuggler's Copter");
    let facts = def_facts(copter);

    // It has a real decision-bearing DSL site by decision_gate.rs's reckoning ...
    assert!(
        !crate::decision_site_walk::row_hits(copter).is_empty(),
        "Smuggler's Copter hits at least one decision_gate.rs row (the incidental DiscardCards) — \
         if this is ever false, the premise of OOS-DP10-9's worked example has changed"
    );
    // ... and still drops its printed `may`, because that site is not may-shaped.
    let may = CHANNELS
        .iter()
        .find(|c| c.id == "may")
        .expect("may channel");
    assert!(!dsl_expresses(&facts, may, &elements["may"]));
}

#[test]
/// The all-faces harvest is load-bearing, not decorative. [`printed_texts`] must actually return
/// more than one text for the corpus's multi-face defs — otherwise the inverse-census fix that
/// introduced it is a no-op wearing a fix's clothes.
///
/// **Measured honestly: widening the oracle axis from `def.oracle_text` to every `oracle_text`
/// key added ZERO offenders.** No effectively-`Complete` def carries a decision marker on a back
/// or Adventure face that its front face does not already carry. That is a fact about today's
/// corpus, not about the walk — a stable count is not evidence that nothing changed (PB-DX26) —
/// and the walk is kept because the next DFC authored is under no obligation to repeat it.
fn t_multi_face_printed_text_is_reached() {
    let defs = all_cards();
    let multi: Vec<&str> = defs
        .iter()
        .filter(|d| printed_texts(d).len() > 1)
        .map(|d| d.name.as_str())
        .collect();
    // Measured 2026-08-12: **19** defs expose more than one `oracle_text`. Floored at 15 — a
    // margin below the measured value, because this population GROWS with authoring and an exact
    // pin would redden on the next DFC. (The first draft of this assertion guessed 20 and went
    // red on the real 19; the number below is measured, not estimated.)
    assert!(
        multi.len() >= 15,
        "only {} defs expose more than one `oracle_text` (measured 19 on 2026-08-12); CardFace \
         carries its own (back_face, adventure_face), so a drop here means the structural harvest \
         has stopped reaching the non-front faces and the front-face-only hole is back",
        multi.len()
    );
    // And the harvest must reach text the front-face read cannot: at least one def whose EXTRA
    // faces contribute words the front face does not.
    let widened = defs.iter().any(|d| {
        let front: BTreeSet<String> = oracle_words(&d.oracle_text).into_iter().collect();
        all_oracle_words(d).into_iter().any(|w| !front.contains(&w))
    });
    assert!(
        widened,
        "no def's non-front faces contribute a single word the front face lacks — the harvest is \
         returning duplicates of the front text rather than reaching the other faces"
    );
}
