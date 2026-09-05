//! CARDS-2 / **SR-37** — the printed-field fidelity gate.
//!
//! Every definition in `all_cards()` is diffed against the printed card, field by field,
//! from a committed fixture. It is the ratchet that CLAUDE.md's Invariant 9 always implied
//! but nothing enforced: `CardDefinition.completeness` gates whether a card's *abilities*
//! are authored, and `validate_deck` refuses a non-`Complete` card, but until this file
//! existed **nothing whatsoever checked that a definition's mana cost, power, toughness or
//! type line matched the card it claims to be**. A def could be `Complete`, pass every
//! test, and cost three mana less than the card printed — which is precisely what
//! `tyrranax_rex` did (`{G}{G}{G}{G}` for a printed `{4}{G}{G}{G}`) until this batch, and
//! what a human found by playing the game rather than by running the suite.
//!
//! ## The three pieces, and why the comparison lives here
//!
//! | piece | what it does |
//! |---|---|
//! | `tools/card-field-dump` | enumerates `all_cards()` (SR-36: never grep the def sources) |
//! | `tools/refresh-card-fidelity-fixture.py` | joins that name list to `cards.sqlite`, copies printed strings **verbatim** |
//! | this file | parses those raw strings and decides equality |
//!
//! `cards.sqlite` is gitignored and does not exist in CI. That is the whole reason the
//! fixture is committed: the gate must run on a machine with no database. It is also why
//! the Python does no normalisation — a pre-normalised fixture would encode a second,
//! unreviewed opinion about what a mana cost *is*, and the two opinions would drift.
//! Everything semantic happens below, once.
//!
//! ## Refreshing the fixture
//!
//! When definitions are added or removed:
//!
//! ```text
//! cargo run -q -p card-field-dump > /tmp/corpus.tsv
//! python3 tools/refresh-card-fidelity-fixture.py \
//!     --corpus /tmp/corpus.tsv --db cards.sqlite \
//!     --out test-data/card-fidelity/printed-fields.tsv
//! ```
//!
//! Refreshing is *not* a way to make a failure go away. The fixture is the printed card;
//! when the gate fails, the definition is wrong. Re-running the script on an unchanged
//! corpus rewrites the same bytes.
//!
//! ## Rules
//!
//! - **R1** — every definition has a fixture row, except the explicitly allowlisted
//!   synthetic test cards.
//! - **R2** — mana costs match structurally (order-free: the fixture's printed string and
//!   the definition's `ManaCost` are both reduced to the same canonical multiset).
//! - **R3** — printed power/toughness match. A non-numeric printed value (`*`, `1+*`) is a
//!   characteristic-defining ability and the definition must carry `None` for that field
//!   (`memory/MEMORY.md`, "Card DSL gotchas").
//! - **R4** — type lines match as *sets* of supertypes / card types / subtypes. Comparing
//!   sets rather than strings sidesteps printed word order entirely.
//! - **R5** — no two definitions share a `name`. `CardRegistry::try_new` already rejects a
//!   duplicate `CardId`, but two files defining the same card under different ids slipped
//!   past it — the corpus carried exactly one such pair when this gate was written.
//! - **R6** — non-vacuity floors, because R1–R5 are all "for every row" assertions and an
//!   empty fixture would satisfy every one of them.
//! - **R8** — a def's `oracle_text` describes the card it names (a similarity floor, because
//!   the corpus predates WotC's 2024 self-reference re-templating). A fictional `oracle_text`
//!   is the documented root cause of this batch's worst incident.
//! - **R7** — costs that live *inside* an ability (bestow, morph, megamorph, disguise, craft)
//!   match the printed clause. R2 sees one field and these are not in it; Boon Satyr's bestow
//!   cost was one of this batch's four headline defects and the gate as first built could not
//!   have caught it. Three more turned up by hand before this rule existed.
//!
//! What a passing gate does NOT assert: that a definition's *abilities* are right. This
//! file checks the four fields that are mechanically checkable against a database. A card
//! can pass every rule here and still be missing half its oracle text — that is what
//! `completeness` is for.

use mtg_engine::{
    AbilityDefinition, CardDefinition, CardType, Completeness, Cost, Effect, HybridMana, ManaColor,
    ManaCost, PhyrexianMana, SubType, SuperType,
};
use std::collections::{BTreeMap, BTreeSet};

const FIXTURE: &str = include_str!("../../../../test-data/card-fidelity/printed-fields.tsv");

/// Definitions that intentionally have no printed card behind them.
///
/// Both are hand-built test fixtures that live in the corpus so integration tests can name
/// a creature with known characteristics. Their in-def comments are the justification; they
/// are listed here by name so that a *typo* in a real card's name — which would also fail
/// to match the database — cannot hide among them.
const SYNTHETIC_ALLOWLIST: &[&str] = &["Poisonous Viper", "Steel Guardian"];

// ── Fixture parsing ───────────────────────────────────────────────────────────

struct Printed {
    mana_cost: String,
    power: String,
    toughness: String,
    type_line: String,
    oracle_text: String,
}

fn fixture() -> BTreeMap<String, Printed> {
    let mut out = BTreeMap::new();
    for line in FIXTURE.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols[0] == "name" {
            continue; // header
        }
        assert_eq!(
            cols.len(),
            6,
            "fixture row has {} columns, expected 6: {line:?}",
            cols.len()
        );
        let prev = out.insert(
            cols[0].to_string(),
            Printed {
                mana_cost: cols[1].to_string(),
                power: cols[2].to_string(),
                toughness: cols[3].to_string(),
                type_line: cols[4].to_string(),
                // The generator escapes newlines as the two characters `\n` so a rules-text
                // paragraph survives a tab-separated cell; undo that here, once.
                oracle_text: cols[5].replace("\\n", "\n"),
            },
        );
        assert!(prev.is_none(), "duplicate fixture row for {:?}", cols[0]);
    }
    out
}

// ── R2: mana cost ─────────────────────────────────────────────────────────────

fn color_letter(c: &ManaColor) -> char {
    match c {
        ManaColor::White => 'W',
        ManaColor::Blue => 'U',
        ManaColor::Black => 'B',
        ManaColor::Red => 'R',
        ManaColor::Green => 'G',
        ManaColor::Colorless => 'C',
    }
}

/// The two halves of a hybrid or Phyrexian-hybrid pip, in a fixed order.
///
/// CR 107.4e: a `{R/G}` pip and a `{G/R}` pip are the same pip — either half pays it. Neither
/// the printed string's ordering nor the definition's argument order carries any meaning, so
/// both are normalised before comparison. Without this the gate would report four false
/// mismatches (Boggart Ram-Gang, Vexing Shusher, Connive // Concoct, Leyline of the Guildpact)
/// that are pure notation.
fn ordered_pair(a: char, b: char) -> (char, char) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// Canonical, order-free rendering of a definition's `ManaCost`.
///
/// `{X}` repeated `x_count` times, then generic as one `{N}`, then WUBRG then `{C}`, then
/// hybrid pips sorted, then Phyrexian pips sorted. Order-free is the right model: `ManaCost`
/// stores counts, not a sequence, so the printed string's ordering is information the
/// definition cannot carry and must not be judged on.
fn canonical_def_cost(cost: &ManaCost) -> String {
    let mut out = String::new();
    for _ in 0..cost.x_count {
        out.push_str("{X}");
    }
    if cost.generic > 0 {
        out.push_str(&format!("{{{}}}", cost.generic));
    }
    for (n, sym) in [
        (cost.white, 'W'),
        (cost.blue, 'U'),
        (cost.black, 'B'),
        (cost.red, 'R'),
        (cost.green, 'G'),
        (cost.colorless, 'C'),
    ] {
        for _ in 0..n {
            out.push_str(&format!("{{{sym}}}"));
        }
    }
    let mut pips: Vec<String> = cost
        .hybrid
        .iter()
        .map(|h| match h {
            HybridMana::ColorColor(a, b) => {
                let (a, b) = ordered_pair(color_letter(a), color_letter(b));
                format!("{{{a}/{b}}}")
            }
            HybridMana::GenericColor(a) => format!("{{2/{}}}", color_letter(a)),
        })
        .collect();
    pips.sort();
    let mut phy: Vec<String> = cost
        .phyrexian
        .iter()
        .map(|p| match p {
            PhyrexianMana::Single(a) => format!("{{{}/P}}", color_letter(a)),
            PhyrexianMana::Hybrid(a, b) => {
                let (a, b) = ordered_pair(color_letter(a), color_letter(b));
                format!("{{{a}/{b}/P}}")
            }
        })
        .collect();
    phy.sort();
    for p in pips.into_iter().chain(phy) {
        out.push_str(&p);
    }
    if out.is_empty() {
        "{0}".to_string()
    } else {
        out
    }
}

/// Reduce a printed Scryfall cost string to the same canonical form.
///
/// Returns `Err` on a symbol this gate does not model, rather than guessing — an unknown
/// symbol must fail loudly, because silently treating it as generic would make some future
/// card's cost check vacuous.
fn canonical_printed_cost(printed: &str) -> Result<String, String> {
    let mut generic = 0u32;
    let mut x_count = 0u32;
    let mut counts: BTreeMap<char, u32> = BTreeMap::new();
    let mut pips: Vec<String> = Vec::new();
    let mut phy: Vec<String> = Vec::new();

    for raw in printed.split('}') {
        let sym = raw.trim_start_matches('{');
        if sym.is_empty() {
            continue;
        }
        if sym == "X" {
            x_count += 1;
        } else if let Ok(n) = sym.parse::<u32>() {
            generic += n;
        } else if sym.len() == 1 && "WUBRGC".contains(sym) {
            *counts.entry(sym.chars().next().unwrap()).or_default() += 1;
        } else if sym.ends_with("/P") {
            // {W/P} or {G/W/P}
            let body: Vec<&str> = sym.trim_end_matches("/P").split('/').collect();
            match body.as_slice() {
                [a] => phy.push(format!("{{{a}/P}}")),
                [a, b] if a.len() == 1 && b.len() == 1 => {
                    let (a, b) = ordered_pair(
                        a.chars().next().expect("len 1"),
                        b.chars().next().expect("len 1"),
                    );
                    phy.push(format!("{{{a}/{b}/P}}"));
                }
                _ => return Err(format!("unmodelled Phyrexian symbol {{{sym}}}")),
            }
        } else if let Some((a, b)) = sym.split_once('/') {
            if a == "2" {
                pips.push(format!("{{2/{b}}}"));
            } else if let (Some(a), Some(b), 1, 1) =
                (a.chars().next(), b.chars().next(), a.len(), b.len())
            {
                let (a, b) = ordered_pair(a, b);
                pips.push(format!("{{{a}/{b}}}"));
            } else {
                return Err(format!("unmodelled hybrid symbol {{{sym}}}"));
            }
        } else {
            return Err(format!("unmodelled mana symbol {{{sym}}}"));
        }
    }

    let mut out = String::new();
    for _ in 0..x_count {
        out.push_str("{X}");
    }
    if generic > 0 {
        out.push_str(&format!("{{{generic}}}"));
    }
    for sym in ['W', 'U', 'B', 'R', 'G', 'C'] {
        for _ in 0..counts.get(&sym).copied().unwrap_or(0) {
            out.push_str(&format!("{{{sym}}}"));
        }
    }
    pips.sort();
    phy.sort();
    for p in pips.into_iter().chain(phy) {
        out.push_str(&p);
    }
    Ok(if out.is_empty() {
        "{0}".to_string()
    } else {
        out
    })
}

// ── R4: type line ─────────────────────────────────────────────────────────────

fn supertype_from_word(w: &str) -> Option<SuperType> {
    Some(match w {
        "Basic" => SuperType::Basic,
        "Legendary" => SuperType::Legendary,
        "Snow" => SuperType::Snow,
        "World" => SuperType::World,
        "Ongoing" => SuperType::Ongoing,
        _ => return None,
    })
}

fn card_type_from_word(w: &str) -> Option<CardType> {
    Some(match w {
        "Artifact" => CardType::Artifact,
        "Battle" => CardType::Battle,
        "Conspiracy" => CardType::Conspiracy,
        "Creature" => CardType::Creature,
        "Dungeon" => CardType::Dungeon,
        "Enchantment" => CardType::Enchantment,
        "Instant" => CardType::Instant,
        // CR 205.2a renamed Tribal to Kindred; older printings in the database still say
        // Tribal and mean the same card type.
        "Kindred" | "Tribal" => CardType::Kindred,
        "Land" => CardType::Land,
        "Phenomenon" => CardType::Phenomenon,
        "Plane" => CardType::Plane,
        "Planeswalker" => CardType::Planeswalker,
        "Scheme" => CardType::Scheme,
        "Sorcery" => CardType::Sorcery,
        "Vanguard" => CardType::Vanguard,
        _ => return None,
    })
}

struct PrintedTypes {
    supertypes: BTreeSet<SuperType>,
    card_types: BTreeSet<CardType>,
    subtypes: BTreeSet<String>,
}

/// Reduce a set of subtypes to the multiset of *words* they print as.
///
/// Both sides of the R4 subtype comparison go through this. It exists because the printed
/// type line is a flat string and this gate has no subtype vocabulary to tokenise it with:
/// most subtypes are one word (`Elf`), a few are two (`Time Lord`), and a land line like
/// `Land — Urza's Cave` is two separate land types that read as one phrase. Comparing words
/// rather than subtype entries makes the gate answer the question it is actually equipped to
/// answer — *does this definition print the same type line as the card?* — without guessing
/// at word boundaries.
///
/// **Known limitation, deliberately accepted**: it cannot tell `SubType("Time Lord")` from
/// `SubType("Time") + SubType("Lord")`. That distinction is real (a type-changing effect
/// matching `Lord` behaves differently) but it is a question about the DSL's representation,
/// not about fidelity to the printed card, and answering it needs a subtype catalogue the
/// repository does not carry. Filed as **OOS-CARDS2-2**.
fn subtype_words<'a>(subtypes: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
    subtypes
        .into_iter()
        .flat_map(|s| s.split_whitespace())
        .map(|w| w.to_string())
        .collect()
}

/// Parse `"Legendary Creature — Elf Druid"` into three sets.
fn parse_type_line(line: &str) -> Result<PrintedTypes, String> {
    // Scryfall uses an em dash; a couple of layouts use a plain hyphen-minus.
    let (head, tail) = match line.split_once('—') {
        Some((h, t)) => (h, Some(t)),
        None => (line, None),
    };
    let mut supertypes = BTreeSet::new();
    let mut card_types = BTreeSet::new();
    for word in head.split_whitespace() {
        if let Some(s) = supertype_from_word(word) {
            supertypes.insert(s);
        } else if let Some(c) = card_type_from_word(word) {
            card_types.insert(c);
        } else {
            return Err(format!("unmodelled type word {word:?} in {line:?}"));
        }
    }
    let subtypes = subtype_words(tail.into_iter().flat_map(|t| t.split_whitespace()));
    Ok(PrintedTypes {
        supertypes,
        card_types,
        subtypes,
    })
}

// ── Shared corpus handle ──────────────────────────────────────────────────────

fn corpus() -> Vec<CardDefinition> {
    mtg_engine::all_cards()
}

/// The face whose characteristics the printed line describes.
///
/// Almost always the definition itself. The exception is a **meld result** (CR 712.5b): the
/// combined permanent gets its own `CardDefinition` that is never in anyone's deck and never
/// cast, exists only to be pointed at by both halves' `meld_pair.melded_card_id`, and carries
/// the melded characteristics on `back_face` with a deliberately empty front. Scryfall prints
/// that card's *melded* line ("Legendary Creature — Eldrazi Ooze", 7/4), so comparing the
/// empty front would report three mismatches that are the design working.
///
/// The exception is recognised **structurally** — by being referenced as some other
/// definition's `melded_card_id` — not by name. A name allowlist would let any def opt out of
/// the gate by being added to a list; this cannot be entered without another card pointing at
/// you, which only a real meld pair does. There is exactly one such definition today
/// (Hanweir, the Writhing Township) and R6 pins that count.
fn meld_result_ids(defs: &[CardDefinition]) -> BTreeSet<String> {
    defs.iter()
        .filter_map(|d| d.meld_pair.as_ref())
        .map(|m| format!("{:?}", m.melded_card_id))
        .collect()
}

struct DefFields {
    mana_cost: Option<ManaCost>,
    power: Option<i32>,
    toughness: Option<i32>,
    supertypes: BTreeSet<SuperType>,
    card_types: BTreeSet<CardType>,
    subtype_words: BTreeSet<String>,
}

fn def_fields(def: &CardDefinition, meld_results: &BTreeSet<String>) -> DefFields {
    if meld_results.contains(&format!("{:?}", def.card_id)) {
        if let Some(face) = def.back_face.as_ref() {
            return DefFields {
                mana_cost: face.mana_cost.clone(),
                power: face.power,
                toughness: face.toughness,
                supertypes: face.types.supertypes.iter().copied().collect(),
                card_types: face.types.card_types.iter().copied().collect(),
                subtype_words: subtype_words(
                    face.types.subtypes.iter().map(|SubType(s)| s.as_str()),
                ),
            };
        }
    }
    DefFields {
        mana_cost: def.mana_cost.clone(),
        power: def.power,
        toughness: def.toughness,
        supertypes: def.types.supertypes.iter().copied().collect(),
        card_types: def.types.card_types.iter().copied().collect(),
        subtype_words: subtype_words(def.types.subtypes.iter().map(|SubType(s)| s.as_str())),
    }
}

fn is_synthetic(name: &str) -> bool {
    SYNTHETIC_ALLOWLIST.contains(&name)
}

// ── R1 ────────────────────────────────────────────────────────────────────────

#[test]
fn r1_every_definition_has_a_printed_row() {
    let printed = fixture();
    let mut orphans = Vec::new();
    for def in corpus() {
        if is_synthetic(&def.name) {
            continue;
        }
        if !printed.contains_key(&def.name) {
            orphans.push(def.name.clone());
        }
    }
    orphans.sort();
    orphans.dedup();
    assert!(
        orphans.is_empty(),
        "{} definition(s) have no printed-field fixture row. Either the name is a typo, or \
         the fixture needs refreshing (see this file's header). Do NOT add them to \
         SYNTHETIC_ALLOWLIST unless the card genuinely does not exist:\n  {}",
        orphans.len(),
        orphans.join("\n  ")
    );

    // The allowlist is a claim about reality; hold it to that. A synthetic that gains a
    // real printing (or a name collision with one) must be noticed, not silently skipped.
    let corpus_names: BTreeSet<String> = corpus().into_iter().map(|d| d.name).collect();
    for name in SYNTHETIC_ALLOWLIST {
        assert!(
            corpus_names.contains(*name),
            "SYNTHETIC_ALLOWLIST names {name:?}, which is not in the corpus — stale entry"
        );
        assert!(
            !printed.contains_key(*name),
            "SYNTHETIC_ALLOWLIST names {name:?}, but the fixture has a printed row for it: \
             the card is real, so remove the allowlist entry and audit the definition"
        );
    }
}

// ── R2 ────────────────────────────────────────────────────────────────────────

#[test]
fn r2_mana_costs_match_printed() {
    let printed = fixture();
    let defs = corpus();
    let melds = meld_result_ids(&defs);
    let mut bad = Vec::new();
    let mut compared = 0usize;
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue; // R1 owns absence
        };
        compared += 1;
        let want = if p.mana_cost == "-" {
            // CR 202.1a: a card with no mana cost cannot be cast without an alternative
            // permission. That is a different card from one costing {0}, so the definition
            // must carry `None`, not `Some(ManaCost::default())`.
            None
        } else {
            match canonical_printed_cost(&p.mana_cost) {
                Ok(c) => Some(c),
                Err(e) => {
                    bad.push(format!("{}: fixture unparseable — {e}", def.name));
                    continue;
                }
            }
        };
        let got = def_fields(def, &melds)
            .mana_cost
            .as_ref()
            .map(canonical_def_cost);
        if got != want {
            bad.push(format!(
                "{}: def {} != printed {} (raw fixture {:?})",
                def.name,
                got.as_deref().unwrap_or("<no mana cost>"),
                want.as_deref().unwrap_or("<no mana cost>"),
                p.mana_cost
            ));
        }
    }
    assert!(compared > 1_700, "R2 compared only {compared} definitions");
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} mana-cost mismatch(es) against the printed card:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

// ── R3 ────────────────────────────────────────────────────────────────────────

/// `Ok(Some(n))` printed numeric; `Ok(None)` printed absent; `Err(())` printed non-numeric
/// (a characteristic-defining ability such as `*` or `1+*`).
fn parse_printed_pt(s: &str) -> Result<Option<i32>, ()> {
    if s == "-" {
        return Ok(None);
    }
    s.parse::<i32>().map(Some).map_err(|_| ())
}

#[test]
fn r3_power_and_toughness_match_printed() {
    let printed = fixture();
    let defs = corpus();
    let melds = meld_result_ids(&defs);
    let mut bad = Vec::new();
    let mut compared = 0usize;
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue;
        };
        compared += 1;
        let fields = def_fields(def, &melds);
        let deck_legal = matches!(def.completeness, Completeness::Complete);
        for (field, raw, got) in [
            ("power", p.power.as_str(), fields.power),
            ("toughness", p.toughness.as_str(), fields.toughness),
        ] {
            match parse_printed_pt(raw) {
                Ok(want) => {
                    if got != want {
                        bad.push(format!(
                            "{} {field}: def {:?} != printed {:?}",
                            def.name, got, want
                        ));
                    }
                }
                Err(()) if deck_legal => {
                    // CDA (CR 604.3): the printed value is not a number, so no fixed value is
                    // correct and a `Complete` definition must express it as one.
                    if got.is_some() {
                        bad.push(format!(
                            "{} {field}: def {:?} but printed {:?} is characteristic-defining \
                             — a Complete definition must carry None and author the CDA",
                            def.name, got, raw
                        ));
                    }
                }
                Err(()) => {
                    // Not deck-legal: `validate_deck` refuses this card, so the placeholder can
                    // never reach a game. This is the ONE field where a non-`Complete` marker
                    // buys an exemption, and only because no correct number exists — a wrong
                    // *mana cost* gets no such pass from R2, since the printed cost is a fact
                    // the definition could simply have copied. The marker is the disclosure
                    // channel, so require it to actually say something.
                    if got.is_some() {
                        let note = match &def.completeness {
                            Completeness::Complete => String::new(),
                            other => format!("{other:?}"),
                        };
                        assert!(
                            note.len() > 20,
                            "{} {field}: carries a placeholder for characteristic-defining {raw:?} \
                             but its completeness marker {note:?} explains nothing",
                            def.name
                        );
                    }
                }
            }
        }
    }
    assert!(compared > 1_700, "R3 compared only {compared} definitions");
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} power/toughness mismatch(es) against the printed card:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

// ── R4 ────────────────────────────────────────────────────────────────────────

#[test]
fn r4_type_lines_match_printed() {
    let printed = fixture();
    let defs = corpus();
    let melds = meld_result_ids(&defs);
    let mut bad = Vec::new();
    let mut compared = 0usize;
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue;
        };
        compared += 1;
        let want = match parse_type_line(&p.type_line) {
            Ok(t) => t,
            Err(e) => {
                bad.push(format!("{}: fixture unparseable — {e}", def.name));
                continue;
            }
        };
        let fields = def_fields(def, &melds);
        let (got_supers, got_types, got_subs) =
            (fields.supertypes, fields.card_types, fields.subtype_words);

        if got_supers != want.supertypes {
            bad.push(format!(
                "{} supertypes: def {:?} != printed {:?} ({:?})",
                def.name, got_supers, want.supertypes, p.type_line
            ));
        }
        if got_types != want.card_types {
            bad.push(format!(
                "{} card types: def {:?} != printed {:?} ({:?})",
                def.name, got_types, want.card_types, p.type_line
            ));
        }
        if got_subs != want.subtypes {
            bad.push(format!(
                "{} subtypes: def {:?} != printed {:?} ({:?})",
                def.name, got_subs, want.subtypes, p.type_line
            ));
        }
    }
    assert!(compared > 1_700, "R4 compared only {compared} definitions");
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} type-line mismatch(es) against the printed card:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

// ── R7: costs that live INSIDE an ability ─────────────────────────────────────

/// The keyword abilities whose cost the DSL stores on an `AbilityDefinition` variant rather
/// than in `CardDefinition.mana_cost`, paired with the word they print as.
///
/// R2 cannot see any of these — it compares one field — and that blind spot is not
/// theoretical: **Boon Satyr's bestow cost was one of the four headline defects of this very
/// batch** (`{4}{G}{G}` for a printed `{3}{G}{G}`) and was found by a human reading the card,
/// not by the gate built to stop exactly this. The batch then found three more by hand
/// (Braided Net's craft, Akroma's and Birchlore Rangers' morph — the last free at `{0}` for a
/// printed `{G}`). Four found by eye in one batch is a class, so it gets a rule.
const ABILITY_COST_KEYWORDS: &[&str] = &[
    "Bestow",
    "Morph",
    "Megamorph",
    "Disguise",
    "Craft",
    "Equip",
    "Fortify",
];

/// Defs whose printed text carries a CR 702.6c **variant** equip cost ("equip
/// [quality] {N}") ahead of the plain line.
///
/// `printed_ability_cost` scans for the first `Equip` occurrence with a
/// brace-delimited cost on its own line, which on these two cards is the variant,
/// not the plain cost the def actually declares. That is not a def error: the
/// variant has no DSL representation at all (`AbilityDefinition::Activated` carries
/// one `cost`, and CR 702.6c restricts the TARGET as well as the cost), so PB-DX26
/// authored the plain line and left the variant unmodelled — filed as
/// `OOS-DX26-2`. Excused HERE, and only for the `Equip` keyword, rather than
/// weakening the scanner for every card.
const EQUIP_VARIANT_COST_DEFS: &[&str] = &["Blackblade Reforged", "Commander's Plate"];

/// Pull the mana cost a printed keyword clause charges, e.g. `Bestow {3}{G}{G}` -> `{3}{G}{G}`.
///
/// Returns `None` when the keyword is absent, and — deliberately — also when it is present
/// without a brace-delimited cost. That covers the real "Morph—Reveal a blue card in your
/// hand." form (CR 702.36b permits a non-mana morph cost) and reminder text that names the
/// keyword in prose. Silence there is correct: this rule compares mana costs and has nothing
/// to say about a cost that is not one.
fn printed_ability_cost(oracle: &str, keyword: &str) -> Option<String> {
    for (idx, _) in oracle.match_indices(keyword) {
        let rest = &oracle[idx + keyword.len()..];
        // PB-DX26: the occurrence must be the WHOLE word. Without this, "Equip"
        // matches inside "Equipped creature has \"{3}, {Q}: …\"" and the scanner
        // reads the GRANTED ability's cost as the equip cost. Measured, not
        // theorised: adding Equip to ABILITY_COST_KEYWORDS reported three
        // "mismatches" (Paradise Mantle, Thornbite Staff, Umbral Mantle) that were
        // all this bug and no def error at all — every one of those three defs
        // declares exactly its printed cost. The leading side is checked too, so a
        // hypothetical "…Equip" suffix cannot match either.
        let leading_ok = oracle[..idx]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        let trailing_ok = rest
            .chars()
            .next()
            .is_none_or(|c| !c.is_ascii_alphanumeric());
        if !leading_ok || !trailing_ok {
            continue;
        }
        // `Craft with artifact {1}{U}` — skip the materials clause to the first brace, but
        // stop at a line break so a later line's cost is never mistaken for this one.
        let line_end = rest.find('\n').unwrap_or(rest.len());
        let line = &rest[..line_end];
        let Some(brace) = line.find('{') else {
            continue;
        };
        // Everything between the keyword and the cost must be cost-less connective tissue.
        // A sentence boundary means this occurrence is prose (reminder text), not the clause.
        if line[..brace].contains('.') || line[..brace].contains('(') {
            continue;
        }
        let cost_run: String = line[brace..]
            .chars()
            .take_while(|c| *c == '{' || *c == '}' || c.is_ascii_alphanumeric() || *c == '/')
            .collect();
        if cost_run.ends_with('}') {
            return Some(cost_run);
        }
    }
    None
}

/// The cost a definition charges for the same keyword, if it declares one.
fn def_ability_cost(def: &CardDefinition, keyword: &str) -> Option<ManaCost> {
    // PB-DX26 fix cycle (review Finding 6): both faces. This walked `def.abilities`
    // only while the batch's own census chained `back_face`, so a DFC Equipment's
    // back-face equip cost was compared against nothing — `None` here makes the
    // caller `continue` SILENTLY, which is the dangerous direction.
    std::iter::once(&def.abilities)
        .chain(def.back_face.iter().map(|f| &f.abilities))
        .flatten()
        .find_map(|a| match (keyword, a) {
            ("Bestow", AbilityDefinition::Bestow { cost }) => Some(cost.clone()),
            ("Morph", AbilityDefinition::Morph { cost }) => Some(cost.clone()),
            ("Megamorph", AbilityDefinition::Megamorph { cost }) => Some(cost.clone()),
            ("Disguise", AbilityDefinition::Disguise { cost }) => Some(cost.clone()),
            ("Craft", AbilityDefinition::Craft { cost, .. }) => Some(cost.clone()),
            // PB-DX26: Equip and Fortify have no dedicated `AbilityDefinition` variant —
            // CR 702.6b/702.67b make both plain activated abilities, so their cost lives
            // in `Activated { cost: Cost::Mana(..) }` beside an `Effect::AttachEquipment`
            // / `Effect::AttachFortification`. Before this, **38 authored equip costs and
            // 1 fortify cost were checked by nothing**: `cards1_equip_target_roster` pins
            // the target requirement and the roster membership, `pb_dx26_attach_keyword_
            // roster` pins that the ability exists, and neither looks at the number. A def
            // charging Equip {1} for a printed Equip {3} sailed past every gate.
            (
                "Equip",
                AbilityDefinition::Activated {
                    cost: Cost::Mana(m),
                    effect: Effect::AttachEquipment { .. },
                    ..
                },
            ) => Some(m.clone()),
            (
                "Fortify",
                AbilityDefinition::Activated {
                    cost: Cost::Mana(m),
                    effect: Effect::AttachFortification { .. },
                    ..
                },
            ) => Some(m.clone()),
            _ => None,
        })
}

// PB-DX26 fix cycle (review Finding L9): the `Equip`/`Fortify` arms above match
// `Effect::AttachEquipment` / `Effect::AttachFortification` FLATLY, not recursively,
// unlike the three walks the batch made recursive. Deliberate, and the reasoning is
// different in each direction:
//
//   * A `Sequence`-nested attach would make this return `None`, and the caller then
//     `continue`s **in silence** — the failure mode is a lost comparison, not a loud
//     one, which is exactly why the roster walks were made recursive.
//   * It is nevertheless safe TODAY and cannot rot unnoticed, because
//     `cards1_equip_target_roster`'s recursive R1 pins the 38-member roster EXACTLY
//     and the per-keyword floor below pins 36 comparisons: a def that nested its
//     attach would stay in R1 (recursive) and drop out of R7's tally (flat), taking
//     the `Equip` floor from 36 to 35 and failing loudly.
//
// So the flat match is covered by a floor rather than by recursion. If that floor is
// ever weakened, make these arms recursive instead.

#[test]
/// R7 — where a card PRINTS a keyword cost and the definition DECLARES that keyword, the two
/// must agree.
///
/// Asymmetric on purpose. A def that declares no `Bestow` ability for a card printing Bestow
/// is *incomplete*, which is `completeness`'s job and not this file's; a def that declares one
/// and charges the wrong number is *wrong*, which is exactly this file's job. Only the second
/// is failed here.
fn r7_ability_embedded_costs_match_printed() {
    let printed = fixture();
    let defs = corpus();
    let mut bad = Vec::new();
    let mut compared = 0usize;
    let mut per_keyword: BTreeMap<&str, usize> = BTreeMap::new();
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue;
        };
        if p.oracle_text == "-" {
            continue;
        }
        for keyword in ABILITY_COST_KEYWORDS {
            if *keyword == "Equip" && EQUIP_VARIANT_COST_DEFS.contains(&def.name.as_str()) {
                // PB-DX26 fix cycle (review Finding L8): the excusal is asserted, not
                // assumed — the same standard `pb_dx26_attach_keyword_roster::r4` applies
                // to its own residual. The entry is only legitimate while the card really
                // does print a CR 702.6c variant line ("equip [quality] {N}") AHEAD of its
                // plain one, which is what confuses the scanner. If the printed text ever
                // stops doing that, the excusal expires here instead of silently hiding a
                // real mismatch forever.
                let lines: Vec<&str> = p.oracle_text.lines().collect();
                let first_equip_line = lines
                    .iter()
                    .position(|l| l.trim_start().starts_with("Equip "))
                    .unwrap_or_else(|| {
                        panic!(
                            "{} is on EQUIP_VARIANT_COST_DEFS but its printed text has no \
                             Equip line at all",
                            def.name
                        )
                    });
                let variant = lines[first_equip_line];
                assert!(
                    !variant.trim_start().starts_with("Equip {"),
                    "{} is excused from R7's Equip comparison because its printed text puts \
                     a CR 702.6c VARIANT cost line ahead of the plain one, which is what \
                     `printed_ability_cost` reads. Its first Equip line is now {variant:?} \
                     — a plain cost — so the excusal no longer applies: remove it from \
                     EQUIP_VARIANT_COST_DEFS and let the comparison run (OOS-DX26-2).",
                    def.name
                );
                continue;
            }
            let (Some(want_raw), Some(got)) = (
                printed_ability_cost(&p.oracle_text, keyword),
                def_ability_cost(def, keyword),
            ) else {
                continue;
            };
            compared += 1;
            *per_keyword.entry(*keyword).or_insert(0usize) += 1;
            let want = match canonical_printed_cost(&want_raw) {
                Ok(c) => c,
                Err(e) => {
                    bad.push(format!("{}: {keyword} cost unparseable — {e}", def.name));
                    continue;
                }
            };
            let got = canonical_def_cost(&got);
            if got != want {
                bad.push(format!(
                    "{} {keyword}: def {got} != printed {want} (raw {want_raw:?})",
                    def.name
                ));
            }
        }
    }
    // MEASURED, not guessed: 9 definitions declared one of the original five variants
    // (Bestow 1, Morph 4, Megamorph 2, Disguise 1, Craft 1). The first draft of this floor
    // was written as `>= 20` from intuition and reddened immediately — which is the whole
    // argument for measuring, made against this file by this file.
    //
    // The number is small because the rule is ASYMMETRIC: it compares only where the def
    // declares the keyword. Two cards print Bestow and only one declares it (Springheart
    // Nantuko is `inert` with a documented blocker), and that gap is `completeness`'s
    // business, not this rule's. Raise the floor when the corpus gains such a def.
    //
    // PB-DX26 fix cycle (review Finding 3): the keyword list grew from five to SEVEN and
    // the aggregate floor was left at 9 — so a regression confined to `Equip` (a broken
    // whole-word guard, a `def_ability_cost` arm that stops matching) would have dropped
    // ~37 comparisons and left this assertion green, with every equip cost unchecked. The
    // aggregate floor is re-measured, and PER-KEYWORD floors are added underneath it,
    // because an aggregate floor cannot see a single keyword going silent. Re-measured by
    // executing this gate: Equip 36 (38 roster members minus the two allowlisted CR 702.6c
    // variant-cost defs), Fortify 1, the original five 9.
    assert!(
        compared >= 46,
        "R7 compared only {compared} ability costs (expected at least 46) — the extraction \
         has stopped matching, which would make this rule silently vacuous"
    );
    for (keyword, floor) in [("Equip", 36usize), ("Fortify", 1)] {
        let got = per_keyword.get(keyword).copied().unwrap_or(0);
        assert!(
            got >= floor,
            "R7 compared only {got} `{keyword}` cost(s), expected at least {floor}. An \
             AGGREGATE floor cannot see one keyword going silent behind the others, which \
             is what this per-keyword floor exists for: before PB-DX26 authored them, 38 \
             equip costs and 1 fortify cost were checked by NO gate at all — the roster \
             gates pin membership and the target requirement and never look at the number."
        );
    }
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} ability-embedded cost mismatch(es) against the printed card:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
}

// ── R8: the oracle text describes the same card ───────────────────────────────

/// Content words of a printed or authored rules text.
///
/// Reminder text in parentheses is dropped (a def may legitimately carry it or not), and the
/// self-reference vocabulary is stopworded, because WotC re-templated every card in 2024 to
/// say "this creature" where the name used to appear and the corpus predates that. Those two
/// rules are what make the comparison about *meaning* rather than about printing era: with
/// them, **the median def scores a perfect 1.00 against its card**.
fn content_words(text: &str) -> BTreeSet<String> {
    const SELF_REFERENCE: &[&str] = &[
        "this",
        "creature",
        "land",
        "artifact",
        "enchantment",
        "permanent",
        "card",
        "it",
        "its",
        "the",
        "a",
        "an",
    ];
    let mut out = BTreeSet::new();
    let mut depth = 0i32;
    let mut word = String::new();
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth = (depth - 1).max(0),
            _ if depth > 0 => {}
            c if c.is_ascii_alphanumeric() || "{}/+-'".contains(c) => {
                word.push(c.to_ascii_lowercase())
            }
            _ => {
                if !word.is_empty() && !SELF_REFERENCE.contains(&word.as_str()) {
                    out.insert(std::mem::take(&mut word));
                } else {
                    word.clear();
                }
            }
        }
    }
    if !word.is_empty() && !SELF_REFERENCE.contains(&word.as_str()) {
        out.insert(word);
    }
    out
}

/// Jaccard overlap of two content-word sets, in hundredths (so the floor is an integer).
fn overlap_pct(a: &BTreeSet<String>, b: &BTreeSet<String>) -> u32 {
    let union = a.union(b).count();
    if union == 0 {
        return 100;
    }
    (a.intersection(b).count() * 100 / union) as u32
}

/// Definitions whose `oracle_text` genuinely does not describe the printed card, or whose
/// low score is an artefact of the fixture rather than a defect. Every entry states which.
///
/// This is a **debt register with a maturity date**, not a licence: the gate fails on anything
/// not listed, and an entry should be deleted the moment its def is repaired. The six marked
/// DEFECT are pre-existing (none was edited by CARDS-2) and are filed as **OOS-CARDS2-10**.
const KNOWN_DIVERGENT_ORACLE_TEXT: &[(&str, &str)] = &[
    // — fixture artefacts, not defects —
    (
        "Cut // Ribbons",
        "ARTEFACT: split card. The def carries both halves; the fixture carries face 0 only.",
    ),
    (
        "Connive // Concoct",
        "ARTEFACT: split card, as Cut // Ribbons.",
    ),
    (
        "Beloved Beggar",
        "ARTEFACT: Disturb DFC. The def's text covers the back face too; the fixture is face 0.",
    ),
    // — real defects, pre-existing, filed as OOS-CARDS2-10 —
    //
    // ALL SIX REPAIRED AND DELETED by PB-DX27 (`scutemob-209`, 2026-08-13): Qarsi Sadist,
    // Scheming Symmetry, Voldaren Epicure, Blasphemous Edict, Delighted Halfling and Flare
    // of Malice each now carry their MCP-verified printed text, so the staleness assertion
    // below (which requires every listed entry to still score < 50) would FAIL on them.
    // That assertion is what forced the deletion — the register is a debt register, not a
    // permanent exemption, and it did its job.
    //
    // Two of the six were `Complete` and deck-legal while under-delivering their printed
    // text, so the repair went past the string in both cases: `voldaren_epicure` gained the
    // dropped "it deals 1 damage to each opponent" clause and stays `Complete`;
    // `qarsi_sadist` was DEMOTED to `partial`, because its missing clause needs a
    // `TriggerCondition::WhenThisExploitsACreature` that does not exist.
    //
    // The three ARTEFACT entries above are deliberately UNTOUCHED — they are fixture
    // shape, not def defects, and repairing a def cannot clear them.
];

// ── R7b: the keyword list is a stated bound, not a hidden floor (`OOS-DX28-1`) ─

/// The `AbilityDefinition` variants R7 actually compares — i.e. the ones `def_ability_cost`
/// has a match arm for.
///
/// `Equip` and `Fortify` are deliberately NOT here: they are not `AbilityDefinition` variants
/// at all (CR 702.6b / 702.67b make both plain activated abilities, so their cost lives in
/// `Activated { cost: Cost::Mana(..) }`), which is why `def_ability_cost` matches them
/// structurally rather than by variant. `r7b` asserts that separation instead of leaving it
/// to a reader to notice.
const SCANNED_MANACOST_VARIANTS: &[&str] = &["Bestow", "Craft", "Disguise", "Megamorph", "Morph"];

/// The printed keyword words in [`ABILITY_COST_KEYWORDS`] that are NOT `AbilityDefinition`
/// variant names, with the reason. See [`SCANNED_MANACOST_VARIANTS`].
const SCANNED_NON_VARIANT_KEYWORDS: &[(&str, &str)] = &[
    (
        "Equip",
        "CR 702.6b -- a plain activated ability. `def_ability_cost` matches \
         `Activated { cost: Cost::Mana(m), effect: Effect::AttachEquipment { .. }, .. }`, \
         not a variant name.",
    ),
    (
        "Fortify",
        "CR 702.67b -- as `Equip`, matched through `Effect::AttachFortification`.",
    ),
];

/// Every `AbilityDefinition` variant that declares a `cost: ManaCost` field and is NOT
/// compared by R7, each with the reason.
///
/// **This list is the deliverable, and its length is the point.** The stage-0 census for
/// `OOS-DX28-1` records `ABILITY_COST_KEYWORDS` as *"a wide floor, not a census"*: R7's own
/// doc says *"four found by eye in one batch is a class, so it gets a rule"*, and the rule
/// covered seven of the class while the DSL declares **40** `cost: ManaCost` variants. The
/// other 35 are printed costs SR-37 does not fidelity-check, and until `r7b` nothing in the
/// tree said so — the floor read like a census, which is exactly the shape the seed is about.
///
/// Two kinds of entry, each labelled:
///
/// * **UNSCANNED GAP** — R7's scanner shape WOULD work (the card prints
///   `<Keyword> {cost}`), and no entry exists. Widening `ABILITY_COST_KEYWORDS` and adding a
///   `def_ability_cost` arm is the fix, one keyword at a time, each with its own measured
///   comparison floor the way `Equip`'s was added.
/// * **STRUCTURAL** — the cost is not printed as a keyword-plus-cost clause at all, so
///   widening the keyword list cannot reach it and a different scanner is needed.
const UNSCANNED_MANACOST_VARIANTS: &[(&str, &str)] = &[
    (
        "Aftermath",
        "STRUCTURAL -- CR 702.127a: the second half is cast from the graveyard for ITS OWN printed mana cost, which sits in that face's cost box and is compared by `r2_mana_costs_match_printed`, not by a keyword clause.",
    ),
    (
        "AltCastAbility",
        "STRUCTURAL -- a generic DSL variant for a card-specific alternative cast cost. There is no printed keyword WORD for `printed_ability_cost` to anchor on, so widening `ABILITY_COST_KEYWORDS` cannot reach it; it needs a different scanner.",
    ),
    (
        "Bloodrush",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Bloodrush {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Buyback",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Buyback {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "ClassLevel",
        "STRUCTURAL -- CR 716.2 prints `Level N {cost}` with no keyword word to anchor on. Reaching it needs a different scanner, not another `ABILITY_COST_KEYWORDS` entry.",
    ),
    (
        "Cleave",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Cleave {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "CommanderNinjutsu",
        "UNSCANNED GAP, with the same naming trap as `MutateCost`: the printed clause is *\"Commander ninjutsu {cost}\"* -- two words, lower-case second word -- so a scanner entry for it is not simply the variant name.",
    ),
    (
        "Cycling",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Cycling {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Disturb",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Disturb {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Echo",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Echo {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Emerge",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Emerge {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Entwine",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Entwine {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Escalate",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Escalate {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Evoke",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Evoke {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Forecast",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Forecast {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Foretell",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Foretell {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Fuse",
        "STRUCTURAL -- CR 702.102a's printed Fuse line carries no cost at all (*\"You may cast one or both halves of this card from your hand\"*); each half is paid at its own face's `mana_cost`, which R2 compares.",
    ),
    (
        "Impending",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Impending {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Kicker",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Kicker {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Madness",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Madness {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Miracle",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Miracle {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "MutateCost",
        "UNSCANNED GAP, and a naming trap worth stating: the PRINTED word is *Mutate* (CR 702.140a) while the VARIANT is `MutateCost`. `ABILITY_COST_KEYWORDS` holds printed words and this list holds variant names, so the two are deliberately not name-identical.",
    ),
    (
        "Ninjutsu",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Ninjutsu {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Offspring",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Offspring {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Outlast",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Outlast {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Overload",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Overload {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Reconfigure",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Reconfigure {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Recover",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Recover {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Replicate",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Replicate {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Scavenge",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Scavenge {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Spectacle",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Spectacle {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Splice",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Splice {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Squad",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Squad {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Surge",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Surge {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
    (
        "Suspend",
        "UNSCANNED GAP -- `ABILITY_COST_KEYWORDS` has no entry for it and `def_ability_cost` has no arm, so a def charging the wrong number for its printed `Suspend {cost}` clause is compared against nothing. R7's scope is a FLOOR, not a census.",
    ),
];

#[test]
/// `OOS-DX28-1` — **R7's keyword list is classified against `pub enum AbilityDefinition`'s
/// own declaration, so its coverage is a stated bound rather than an unstated floor.**
///
/// ## What this row does and does not claim
///
/// It does NOT claim the 35 unscanned variants are fine. It claims the opposite, in writing,
/// per variant: each is a printed mana cost that R7 compares against nothing, and each is
/// named so that closing one is a deliberate act and ADDING a 41st `cost: ManaCost` variant
/// is a red test rather than a silent widening of the gap.
///
/// ## Why the seed applies here
///
/// `ABILITY_COST_KEYWORDS` is a hand-maintained fingerprint over a declaration that grows.
/// Nothing compared the two, so the class it covers could shrink in relative terms with every
/// DSL addition while every test stayed green — the same end state as `TARGET_FILTER_FIELDS`,
/// reached by growth of the SUBJECT rather than of the fingerprint.
///
/// **"In silence" is precise, not rhetorical.** Planting a 41st `cost: ManaCost` variant was
/// EXECUTED: it reddens `ability_definition_registry::all_ability_definitions_covers_every_variant`
/// (the variant exists and is unclassified there) and the two wire gates. Neither of those
/// says anything about R7 — `r7_ability_embedded_costs_match_printed` itself stayed **GREEN**
/// under the plant, comparing the same five keywords it always did. What was silent is the
/// widening of R7's blind spot, and that is what this row measures.
///
/// ## The exposure figures are PRINTED, not transcribed
///
/// PB-DX8's rule. The per-variant corpus exposure below (how many defs declare the variant,
/// and how many of those also print a brace-delimited cost on the keyword's own line, i.e.
/// how many R7 could compare today if the keyword were added) is measured at run time and
/// printed. Reading it off a comment written months ago is how a census becomes a claim.
fn r7b_every_manacost_bearing_ability_variant_is_scanned_or_excluded() {
    // **STATED RESIDUAL — two consistent lists can shrink TOGETHER.** The adversarial pass
    // removed a keyword from the scanned list AND from the classification in one edit; the
    // partition still holds and this gate stays green. What actually holds the line is `r7`'s
    // aggregate floor (exactly 46, zero slack) plus the per-keyword floors, which exist only
    // for `Equip` and `Fortify`. So this gate proves the classification is TOTAL over the
    // declaration; it does not prove any particular keyword is still SCANNED. A per-keyword
    // floor for the other three scanned keywords would close it and is not taken here.
    let variant_fields = crate::pb_dx57_declared_source::declared_enum_variant_fields(
        crate::pb_dx57_declared_source::CARD_DEFINITION_RS,
        "AbilityDefinition",
    );
    assert!(
        variant_fields.len() >= 50,
        "the `pub enum AbilityDefinition` parse returned only {} variants — the parser is \
         broken and every assertion below is vacuous",
        variant_fields.len()
    );

    // The declaration side: every variant whose payload declares a `cost` field typed
    // `ManaCost`. `Activated` (`Cost`), `CumulativeUpkeep` (`CumulativeUpkeepCost`) and
    // `LoyaltyAbility` (`LoyaltyCost`) declare a `cost` of a DIFFERENT type and are therefore
    // outside this row by construction, not by omission — the parser keeps each variant's
    // field TYPE, so this is decided rather than assumed.
    let src = crate::pb_dx57_declared_source::read_workspace_file(
        crate::pb_dx57_declared_source::CARD_DEFINITION_RS,
    );
    let declared: BTreeSet<String> = variant_fields
        .iter()
        .filter(|(name, fields)| fields.contains("cost") && variant_declares_mana_cost(&src, name))
        .map(|(name, _)| name.clone())
        .collect();
    assert!(
        declared.len() >= 30,
        "only {} AbilityDefinition variant(s) were found to declare `cost: ManaCost` — the \
         type match is broken and this row's population has collapsed",
        declared.len()
    );

    let scanned: BTreeSet<String> = SCANNED_MANACOST_VARIANTS
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let unscanned: BTreeSet<String> = UNSCANNED_MANACOST_VARIANTS
        .iter()
        .map(|(n, _)| (*n).to_string())
        .collect();

    let both: Vec<&String> = scanned.intersection(&unscanned).collect();
    assert!(
        both.is_empty(),
        "variant(s) classified as both scanned and unscanned: {both:?}"
    );

    let classified: BTreeSet<String> = scanned.union(&unscanned).cloned().collect();
    assert_eq!(
        classified,
        declared,
        "`OOS-DX28-1`: the `cost: ManaCost`-bearing `AbilityDefinition` variants are no \
         longer exactly the classified set.\n  DECLARED but unclassified: {:?}\n    → this is \
         a NEW printed mana cost that SR-37 does not check. Either add it to \
         `ABILITY_COST_KEYWORDS` with a `def_ability_cost` arm and put it in \
         SCANNED_MANACOST_VARIANTS, or add it to UNSCANNED_MANACOST_VARIANTS with the reason. \
         Do not delete this assertion: before it existed, a 41st variant widened R7's blind \
         spot in silence.\n  CLASSIFIED but undeclared: {:?}\n    → a rename or a removal; if \
         it is in SCANNED_MANACOST_VARIANTS then `def_ability_cost`'s arm for it no longer \
         compiles against anything real.",
        declared.difference(&classified).collect::<Vec<_>>(),
        classified.difference(&declared).collect::<Vec<_>>()
    );

    // Every scanned variant really has an `ABILITY_COST_KEYWORDS` entry, and every keyword
    // that is NOT a variant is accounted for. This is the half that keeps the two lists from
    // drifting in the other direction: a keyword removed from ABILITY_COST_KEYWORDS while
    // SCANNED_MANACOST_VARIANTS still names it would silently stop comparing that keyword.
    let keywords: BTreeSet<&str> = ABILITY_COST_KEYWORDS.iter().copied().collect();
    let non_variant_keywords: BTreeSet<&str> = SCANNED_NON_VARIANT_KEYWORDS
        .iter()
        .map(|(k, _)| *k)
        .collect();
    let expected_keywords: BTreeSet<&str> = SCANNED_MANACOST_VARIANTS
        .iter()
        .copied()
        .chain(non_variant_keywords.iter().copied())
        .collect();
    assert_eq!(
        keywords,
        expected_keywords,
        "ABILITY_COST_KEYWORDS and the r7b classification have drifted. In the keyword list \
         only: {:?}; in the classification only: {:?}",
        keywords.difference(&expected_keywords).collect::<Vec<_>>(),
        expected_keywords.difference(&keywords).collect::<Vec<_>>()
    );
    // ...and the non-variant keywords really are not variants, which is the whole reason they
    // are matched structurally instead.
    for (kw, _) in SCANNED_NON_VARIANT_KEYWORDS {
        assert!(
            !variant_fields.contains_key(*kw),
            "`{kw}` is now a declared `AbilityDefinition` variant, so `def_ability_cost`'s \
             structural `Activated {{ .. }}` arm for it may no longer be the right shape — \
             move it to SCANNED_MANACOST_VARIANTS and give it a variant arm"
        );
    }

    for (name, why) in UNSCANNED_MANACOST_VARIANTS
        .iter()
        .chain(SCANNED_NON_VARIANT_KEYWORDS)
    {
        assert!(
            why.len() > 40,
            "the r7b classification entry for `{name}` carries no real reason: {why:?}"
        );
    }

    // ── The exposure measurement, PRINTED rather than transcribed (PB-DX8's rule).
    let printed = fixture();
    let defs = corpus();
    let mut declaring: BTreeMap<&str, usize> = BTreeMap::new();
    let mut comparable: BTreeMap<&str, usize> = BTreeMap::new();
    for def in &defs {
        let json = serde_json::to_value(def).expect("CardDefinition serializes");
        for (name, _) in UNSCANNED_MANACOST_VARIANTS {
            if !count_variant_key(&json, name) {
                continue;
            }
            *declaring.entry(*name).or_insert(0) += 1;
            if let Some(p) = printed.get(&def.name) {
                if p.oracle_text != "-" && printed_ability_cost(&p.oracle_text, name).is_some() {
                    *comparable.entry(*name).or_insert(0) += 1;
                }
            }
        }
    }
    println!(
        "r7b: {} AbilityDefinition variants declare `cost: ManaCost`; R7 compares {} of them \
         (plus the {} non-variant keywords Equip/Fortify). Corpus exposure of the {} \
         UNSCANNED variants — `declaring` = defs carrying the variant, `comparable` = defs \
         whose printed text ALSO yields a cost for a scan keyed on the variant name, i.e. \
         what a naive widening would compare today:",
        declared.len(),
        scanned.len(),
        non_variant_keywords.len(),
        unscanned.len()
    );
    for (name, _) in UNSCANNED_MANACOST_VARIANTS {
        println!(
            "  {name:<20} declaring={:<4} comparable={}",
            declaring.get(name).copied().unwrap_or(0),
            comparable.get(name).copied().unwrap_or(0)
        );
    }
    let total_declaring: usize = declaring.values().sum();
    assert!(
        total_declaring > 0,
        "no corpus def declares ANY of the 35 unscanned `cost: ManaCost` variants. That is \
         not a clean bill of health — it means `count_variant_key`'s walk stopped finding \
         variants, and the exposure table above is measuring nothing."
    );
}

/// Does `pub enum AbilityDefinition`'s declaration of `variant` type its `cost` field as
/// `ManaCost`?
///
/// Kept separate from the parsed field map because that map keeps NAMES, and `Activated`
/// (`Cost`), `CumulativeUpkeep` (`CumulativeUpkeepCost`) and `LoyaltyAbility` (`LoyaltyCost`)
/// all declare a field called `cost` of a different type. Deciding by name alone would put
/// three variants in this row's population that do not belong there.
fn variant_declares_mana_cost(src: &str, variant: &str) -> bool {
    // Anchor on the variant header, then look for `cost: ManaCost` before the next variant
    // header at the same nesting depth. The variant bodies here are small and flat, so a
    // brace-matched window from the header is enough.
    let Some(at) = src.find(&format!("\n    {variant} {{")) else {
        return false;
    };
    let body_start = src[at..].find('{').map(|i| at + i + 1).unwrap_or(at);
    let mut depth = 1usize;
    let mut end = body_start;
    for (i, ch) in src[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = body_start + i;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &src[body_start..end];
    body.lines()
        .map(str::trim)
        .any(|l| l.starts_with("cost:") && l.contains("ManaCost"))
}

/// Does this serialized `CardDefinition` carry `key` as an object key anywhere?
///
/// A local recursive walk rather than `decision_site_walk::def_contains_variant`, because
/// this one must NOT match a bare unit-variant string: every name it is asked about is a
/// struct variant, and the prose suppression `def_contains_variant` needs for unit variants
/// is irrelevant here.
fn count_variant_key(v: &serde_json::Value, key: &str) -> bool {
    match v {
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(k, child)| k == key || count_variant_key(child, key)),
        serde_json::Value::Array(items) => items.iter().any(|i| count_variant_key(i, key)),
        _ => false,
    }
}

#[test]
/// R8 — a definition's `oracle_text` must describe the card it names.
///
/// This rule exists because a fictional `oracle_text` is the **documented root cause** of this
/// batch's worst incident: a repair pass authored three abilities into `braided_net` after
/// being briefed from that file's own stale text. Nothing checked the field, so nothing
/// stopped the fiction propagating from a comment into working code.
///
/// It is a similarity floor, not equality, and that is a measured choice rather than a
/// concession. Comparing the strings outright fails on **315** definitions, almost all of them
/// WotC's 2024 self-reference re-templating and reminder-text churn — a gate that fails on a
/// sixth of the corpus for spelling gets turned off. With reminder text dropped and the
/// self-reference vocabulary stopworded, the **median definition scores 100**, and only 9 of
/// 1,776 comparable definitions fall below 50. Six of those nine are real defects the corpus
/// did not know it had.
///
/// Definitions with no content words on one side (vanilla creatures, basic lands) are skipped:
/// there is nothing to compare, not a mismatch.
///
/// **What this floor structurally cannot see, measured rather than guessed.** A reviewer
/// cross-paired every def's printed text against 40 random other cards — 71,003 pairings — and
/// **0.196% of completely wrong pairings still score 50 or more**. They concentrate in
/// template-identical cards: the shocklands score **100** against each other, because the only
/// thing distinguishing them is the mana symbol inside their reminder text, which this function
/// drops on purpose; Elvish Mystic vs Fyndhorn Elves and Crucible of Worlds vs Ramunap
/// Excavator behave the same way. R2 and R4 catch every one of those (different costs, different
/// type lines), so the corpus is covered — but this rule alone would not, and a future reader
/// should not read "median 100" as "any wrong text fails here".
///
/// One process note, because it is the batch's own lesson turned on itself: the floor of 50 was
/// chosen from a throwaway Python approximation of this function, which scored
/// `delighted_halfling` at exactly 50 and `doom_blade` at 50 — so the first draft of the
/// register listed the real defect as a miss and the spelling difference as needing an
/// exemption. **The implementation that ships scores them 47 and 50**, i.e. it catches the
/// defect and needs no entry for the hyphen. Both notes were wrong in the same direction, and
/// only running the real thing settled it.
fn r8_oracle_text_describes_the_same_card() {
    let printed = fixture();
    let defs = corpus();
    let mut bad = Vec::new();
    let mut compared = 0usize;
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue;
        };
        if p.oracle_text == "-" {
            continue;
        }
        let (a, b) = (
            content_words(&def.oracle_text),
            content_words(&p.oracle_text),
        );
        if a.is_empty() || b.is_empty() {
            continue;
        }
        compared += 1;
        let score = overlap_pct(&a, &b);
        let listed = KNOWN_DIVERGENT_ORACLE_TEXT
            .iter()
            .any(|(name, _)| *name == def.name);
        if score < 50 && !listed {
            bad.push(format!(
                "{}: overlap {score} — def {:?}",
                def.name, def.oracle_text
            ));
        }
    }
    assert!(
        compared > 1_700,
        "R8 compared only {compared} definitions — the word extraction has stopped matching"
    );
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} definition(s) carry an `oracle_text` that does not describe their card. This is \
         how `braided_net` got three invented abilities authored into it — the fiction lived \
         in the def until someone believed it. Repair the text against Scryfall, or add an \
         entry to KNOWN_DIVERGENT_ORACLE_TEXT stating which of the two it is:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
    // Every listed entry must still be needed. A def that gets repaired and stays on the list
    // turns the register into a permanent exemption.
    for (name, _) in KNOWN_DIVERGENT_ORACLE_TEXT {
        let Some(def) = defs.iter().find(|d| d.name == *name) else {
            panic!("KNOWN_DIVERGENT_ORACLE_TEXT names {name:?}, which is not in the corpus");
        };
        let Some(p) = printed.get(*name) else {
            continue;
        };
        let score = overlap_pct(
            &content_words(&def.oracle_text),
            &content_words(&p.oracle_text),
        );
        assert!(
            score < 50,
            "{name} now scores {score} and no longer diverges — delete its \
             KNOWN_DIVERGENT_ORACLE_TEXT entry"
        );
    }
}

// ── R5 ────────────────────────────────────────────────────────────────────────

#[test]
fn r5_no_two_definitions_share_a_name() {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for def in corpus() {
        seen.entry(def.name.clone())
            .or_default()
            .push(format!("{:?}", def.card_id));
    }
    let dupes: Vec<String> = seen
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|(name, ids)| format!("{name}: {}", ids.join(", ")))
        .collect();
    assert!(
        dupes.is_empty(),
        "{} card name(s) defined more than once. `CardRegistry::try_new` only rejects a \
         duplicate CardId, so two files defining the same card under different ids build \
         cleanly and both ship:\n  {}",
        dupes.len(),
        dupes.join("\n  ")
    );
}

// ── R6 ────────────────────────────────────────────────────────────────────────

#[test]
fn r6_non_vacuity_floors() {
    let printed = fixture();
    assert!(
        printed.len() > 1_750,
        "fixture has only {} rows — a truncated fixture would make R2/R3/R4 vacuous",
        printed.len()
    );
    assert!(
        corpus().len() > 1_750,
        "corpus has only {} definitions",
        corpus().len()
    );
    assert_eq!(
        SYNTHETIC_ALLOWLIST.len(),
        2,
        "the synthetic allowlist grew — every entry is a card that does not exist, so \
         each addition needs a human to confirm that claim"
    );

    // The meld exception is the one place R2/R3/R4 read a face other than the definition's
    // own. Pin its size, because it is the shape a future def could quietly hide inside.
    let defs = corpus();
    let melds = meld_result_ids(&defs);
    assert_eq!(
        melds.len(),
        1,
        "expected exactly one meld result (Hanweir, the Writhing Township); found {melds:?}. \
         A new meld pair is fine — confirm its result definition really is a never-cast shell \
         with its characteristics on back_face, then update this count."
    );
    let melded: Vec<&CardDefinition> = defs
        .iter()
        .filter(|d| melds.contains(&format!("{:?}", d.card_id)))
        .collect();
    assert_eq!(melded.len(), 1, "meld result id resolves to no definition");
    assert!(
        melded[0].back_face.is_some(),
        "{}: named as a meld result but carries no back_face — the melded characteristics \
         have nowhere to live and def_fields would silently compare the empty front",
        melded[0].name
    );

    // The canonicalisers must actually canonicalise: two orderings of the same printed
    // cost reduce alike, and two different costs do not. Without this, a bug that made
    // `canonical_printed_cost` return a constant would leave R2 green and empty.
    assert_eq!(
        canonical_printed_cost("{4}{G}{G}{G}").unwrap(),
        canonical_printed_cost("{G}{G}{4}{G}").unwrap()
    );
    assert_ne!(
        canonical_printed_cost("{4}{G}{G}{G}").unwrap(),
        canonical_printed_cost("{G}{G}{G}{G}").unwrap()
    );
    assert_eq!(canonical_printed_cost("{X}{2}{U}").unwrap(), "{X}{2}{U}");
    assert_eq!(canonical_printed_cost("").unwrap(), "{0}");
    assert_eq!(
        canonical_printed_cost("{2/W}{W/U}{U/P}").unwrap(),
        "{2/W}{U/W}{U/P}"
    );
    // CR 107.4e: {R/G} and {G/R} are one pip, printed two ways. Both reduce alike.
    assert_eq!(
        canonical_printed_cost("{R/G}{R/G}").unwrap(),
        canonical_printed_cost("{G/R}{G/R}").unwrap()
    );
    assert!(canonical_printed_cost("{Q}").is_err());
    // Multi-word subtypes reduce to the same words whether written as one entry or two.
    assert_eq!(
        subtype_words(["Urza's Cave"]),
        subtype_words(["Urza's", "Cave"])
    );
    assert_ne!(subtype_words(["Urza's Cave"]), subtype_words(["Cave"]));

    let cost = ManaCost {
        generic: 4,
        green: 3,
        ..Default::default()
    };
    assert_eq!(canonical_def_cost(&cost), "{4}{G}{G}{G}");
    let hybrid = ManaCost {
        hybrid: vec![HybridMana::ColorColor(ManaColor::Red, ManaColor::Green)],
        ..Default::default()
    };
    assert_eq!(canonical_def_cost(&hybrid), "{G/R}");
    assert_eq!(canonical_def_cost(&ManaCost::default()), "{0}");

    let t = parse_type_line("Legendary Enchantment Creature — Elf Druid").unwrap();
    assert_eq!(t.supertypes, BTreeSet::from([SuperType::Legendary]));
    assert_eq!(
        t.card_types,
        BTreeSet::from([CardType::Enchantment, CardType::Creature])
    );
    assert_eq!(
        t.subtypes,
        BTreeSet::from(["Elf".to_string(), "Druid".to_string()])
    );
    assert!(parse_type_line("Creature — Elf")
        .unwrap()
        .supertypes
        .is_empty());
    assert_eq!(parse_printed_pt("-"), Ok(None));
    assert_eq!(parse_printed_pt("4"), Ok(Some(4)));
    assert_eq!(parse_printed_pt("*"), Err(()));
    assert_eq!(parse_printed_pt("1+*"), Err(()));
}
