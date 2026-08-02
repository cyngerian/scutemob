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
    AbilityDefinition, CardDefinition, CardType, Completeness, HybridMana, ManaColor, ManaCost,
    PhyrexianMana, SubType, SuperType,
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
const ABILITY_COST_KEYWORDS: &[&str] = &["Bestow", "Morph", "Megamorph", "Disguise", "Craft"];

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
    def.abilities.iter().find_map(|a| match (keyword, a) {
        ("Bestow", AbilityDefinition::Bestow { cost }) => Some(cost.clone()),
        ("Morph", AbilityDefinition::Morph { cost }) => Some(cost.clone()),
        ("Megamorph", AbilityDefinition::Megamorph { cost }) => Some(cost.clone()),
        ("Disguise", AbilityDefinition::Disguise { cost }) => Some(cost.clone()),
        ("Craft", AbilityDefinition::Craft { cost, .. }) => Some(cost.clone()),
        _ => None,
    })
}

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
    for def in &defs {
        let Some(p) = printed.get(&def.name) else {
            continue;
        };
        if p.oracle_text == "-" {
            continue;
        }
        for keyword in ABILITY_COST_KEYWORDS {
            let (Some(want_raw), Some(got)) = (
                printed_ability_cost(&p.oracle_text, keyword),
                def_ability_cost(def, keyword),
            ) else {
                continue;
            };
            compared += 1;
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
    // MEASURED, not guessed: 9 definitions declare one of these five variants today
    // (Bestow 1, Morph 4, Megamorph 2, Disguise 1, Craft 1). The first draft of this floor
    // was written as `>= 20` from intuition and reddened immediately — which is the whole
    // argument for measuring, made against this file by this file.
    //
    // The number is small because the rule is ASYMMETRIC: it compares only where the def
    // declares the keyword. Two cards print Bestow and only one declares it (Springheart
    // Nantuko is `inert` with a documented blocker), and that gap is `completeness`'s
    // business, not this rule's. Raise the floor when the corpus gains such a def.
    assert!(
        compared >= 9,
        "R7 compared only {compared} ability costs — the extraction has stopped matching, \
         which would make this rule silently vacuous"
    );
    bad.sort();
    assert!(
        bad.is_empty(),
        "{} ability-embedded cost mismatch(es) against the printed card:\n  {}",
        bad.len(),
        bad.join("\n  ")
    );
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
