//! The local card-image store, its two routes, and the deck-time prefetch.
//!
//! Owner ruling 2026-10-12 (`memory/decisions.md`, `docs/play-surface-v3-brief.md` §5):
//! the browser never calls Scryfall on the v3 surface. Card images live in a local,
//! gitignored store that is filled **when a game's decks are resolved** (or ahead of
//! time with `--prefetch-images`), at the three sizes the UI uses. Play reads the
//! store only: a miss is a 404 and the client shows its text chip.
//!
//! # The route never fetches
//!
//! [`get_card_image`] reads one file and answers. It has no handle on the network
//! half of this module at all — the fetcher lives behind [`ImageService::enqueue`],
//! which only `api::post_game` calls. A store miss is a defect to fix at deck-add
//! time, never a live fetch.
//!
//! # Hidden information (Architecture Invariant 7)
//!
//! Neither route reads the game session. `GET /api/card-image` maps a card NAME the
//! client already holds to bytes on disk; any client can ask for any name, and the
//! answer says nothing about WHERE a card is (which seat, which zone).
//!
//! **Known consequence of the deck-time fill (owner ruling 2026-10-12).** The store
//! is filled from every seat's deck, bot seats included. So on a store that was not
//! already full, `GET /api/card-image?name=X` answering 200 rather than 404 reveals
//! that X is in the UNION of the decks this store has been filled for. It does not
//! say which seat holds X, it is diluted as the store accumulates games (a 200 then
//! only means "some table dealt on this store had X"), and it takes deliberate
//! probing of names by the local player — the page itself only asks for names it was
//! already shipped. Two ways to close it, **awaiting an owner ruling, neither
//! implemented**: (a) prefetch the whole card pool up front, so a 200 says nothing;
//! (b) accept it for a local single-human table.
//!
//! The status route reports COUNTS only. `failed` is the number of cards that could
//! not be downloaded; their names (cards of the table's decks, bot seats included —
//! a decklist is hidden information) go to stderr, the server's own log, with the
//! reason, and never into a response body. Queued names are not reported anywhere.
//!
//! # Store layout
//!
//! ```text
//! <root>/small/<slug>.jpg     table size   (Scryfall `small`,  146x204)
//! <root>/normal/<slug>.jpg    hover size   (Scryfall `normal`, 488x680)
//! <root>/large/<slug>.jpg     card window  (Scryfall `large`,  672x936)
//! ```
//!
//! `<slug>` is [`slug`]: a readable ASCII stem plus a 16-hex-digit FNV-1a hash of the
//! exact name, so it can only ever contain `[a-z0-9-]` (no separator, no dot — it
//! cannot leave the root) and two names that differ only in punctuation cannot share
//! a file.
//!
//! # Where a card's images come from (`scutemob-270`)
//!
//! `cards.sqlite` already holds every card's Scryfall id, and the image CDN's paths
//! are a pure function of that id ([`cdn_image_url`]). So a card whose name the
//! sqlite knows ([`CardIds`], [`SqliteIds`]) is downloaded from `cards.scryfall.io`
//! alone: **no `api.scryfall.com` request at all**. The rate-limited named lookup
//! ([`named_card_url`]) remains only as the fallback — for a name the sqlite does
//! not hold, when no sqlite could be opened, and (once per card) when an id-derived
//! path answers 404. The status route counts both routes.
//!
//! # Retry
//!
//! A card that failed for a reason worth trying again (anything but "Scryfall knows
//! no card by this name") stays in the **retry queue**. It is tried again when a
//! later deal names it, and `POST /api/card-image/retry` re-queues the whole retry
//! queue at once. The status route reports the queue's size.
//!
//! # Testability
//!
//! Planning what to fetch ([`missing_versions`]), the slug, the id to CDN-URL
//! derivation ([`cdn_image_url`]), the Scryfall JSON to image-URL extraction
//! ([`image_urls`]), the backoff schedule ([`backoff_delay`]) and the decklist
//! parser are pure. The whole fetch procedure ([`ensure_card_via`]) is written
//! against the [`Net`] and [`CardIds`] traits. [`HttpNet`] is the only code in this
//! crate that talks to the network; tests drive the procedure — and the background
//! worker — with a scripted fake and never construct `HttpNet`.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use axum::extract::{Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use mtg_simulator::{DeckSource, LocalGameConfig};
use serde::Serialize;

use crate::api::{ApiError, ApiFailure};
use crate::session::SharedState;

// ── Versions and the slug ─────────────────────────────────────────────────────

/// The three image sizes the UI uses. Exactly these; anything else is a 400.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Version {
    /// The table.
    Small,
    /// The hover magnifier.
    Normal,
    /// The card window.
    Large,
}

impl Version {
    pub const ALL: [Version; 3] = [Version::Small, Version::Normal, Version::Large];

    /// The wire spelling, the store directory name and Scryfall's `image_uris` key.
    pub fn as_str(self) -> &'static str {
        match self {
            Version::Small => "small",
            Version::Normal => "normal",
            Version::Large => "large",
        }
    }

    pub fn parse(text: &str) -> Option<Version> {
        Version::ALL.into_iter().find(|v| v.as_str() == text)
    }
}

/// Longest readable stem kept in a slug; the hash carries the identity.
const SLUG_STEM_MAX: usize = 60;

/// 64-bit FNV-1a over the exact bytes of `name`. Stable across builds and
/// platforms (unlike `std`'s `DefaultHasher`, whose algorithm is unspecified), which
/// a file name on disk needs.
fn fnv1a64(name: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in name.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// File stem for a card name: `<readable-stem>-<16 hex digits>`.
///
/// The stem is the name lowercased with every run of non-`[a-z0-9]` characters
/// collapsed to one `-` (so `Atraxa, Praetors' Voice` reads `atraxa-praetors-voice`),
/// capped at [`SLUG_STEM_MAX`]. That stem alone collides (`Fire // Ice` and
/// `Fire-Ice`, or two accented names that differ only in the accent), so the hash of
/// the EXACT name is appended: equal slugs mean equal names, up to a 64-bit
/// collision. The output alphabet is `[a-z0-9-]`, so no input can produce a path
/// separator, a dot or an absolute path.
pub fn slug(name: &str) -> String {
    let mut stem = String::new();
    let mut pending_dash = false;
    for ch in name.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_lowercase() || lower.is_ascii_digit() {
            if pending_dash && !stem.is_empty() {
                stem.push('-');
            }
            pending_dash = false;
            stem.push(lower);
        } else {
            pending_dash = true;
        }
        if stem.len() >= SLUG_STEM_MAX {
            break;
        }
    }
    if stem.is_empty() {
        stem.push_str("card");
    }
    format!("{stem}-{:016x}", fnv1a64(name))
}

// ── The store ─────────────────────────────────────────────────────────────────

/// A directory of card images. See the module doc for the layout.
#[derive(Clone, Debug)]
pub struct ImageStore {
    root: PathBuf,
}

impl ImageStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        ImageStore { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Where `name` at `version` lives. Always strictly inside the root.
    pub fn path_for(&self, name: &str, version: Version) -> PathBuf {
        self.root
            .join(version.as_str())
            .join(format!("{}.jpg", slug(name)))
    }

    pub fn has(&self, name: &str, version: Version) -> bool {
        self.path_for(name, version).is_file()
    }

    /// Write `bytes` for `name` at `version` through a temp file and a rename, so a
    /// concurrent read sees the old state (absent) or the whole image, never a
    /// half-written one.
    pub fn write_atomic(&self, name: &str, version: Version, bytes: &[u8]) -> std::io::Result<()> {
        let dest = self.path_for(name, version);
        if let Some(dir) = dest.parent() {
            std::fs::create_dir_all(dir)?;
        }
        // The `.part` suffix keeps a leftover from a killed process from ever being
        // served: the route only opens `<slug>.jpg`.
        let tmp = dest.with_extension(format!("jpg.{}.part", std::process::id()));
        std::fs::write(&tmp, bytes)?;
        std::fs::rename(&tmp, &dest).inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
    }
}

/// The pure planner: which of the three versions of `name` the store lacks.
/// Empty means "nothing to fetch for this card".
pub fn missing_versions(store: &ImageStore, name: &str) -> Vec<Version> {
    Version::ALL
        .into_iter()
        .filter(|v| !store.has(name, *v))
        .collect()
}

// ── Scryfall: URLs, JSON, backoff ─────────────────────────────────────────────

/// Identifies this tool to Scryfall, as their API guidelines require.
pub const USER_AGENT: &str = "scutemob-play-server/0.1 (local card-image prefetch)";

/// Pause before every request to `api.scryfall.com`. Scryfall asks for 50-100 ms
/// between API calls; this stays clear of the upper figure.
pub const API_SPACING: Duration = Duration::from_millis(120);

/// Least time between the STARTS of two id-derived CDN downloads ([`Net::space`]):
/// at most ten requests a second, the rate Scryfall asks API clients to stay under.
pub const CDN_SPACING: Duration = Duration::from_millis(100);

/// How many times one URL is tried before the card is recorded as failed.
pub const MAX_ATTEMPTS: u32 = 5;

/// The only host an image is ever downloaded from. An `image_uris` entry pointing
/// anywhere else is ignored rather than followed.
const IMAGE_CDN_PREFIX: &str = "https://cards.scryfall.io/";

/// Percent-encode `text` as one query-string value (RFC 3986 unreserved set kept).
fn percent_encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// The ONE rate-limited call per card: the card object, as JSON, by exact name.
pub fn named_card_url(name: &str) -> String {
    format!(
        "https://api.scryfall.com/cards/named?exact={}",
        percent_encode(name)
    )
}

/// Whether `id` has the shape of a Scryfall id: a lowercase hyphenated UUID
/// (`8-4-4-4-12` hex digits). The id comes out of a file on disk; anything else
/// must not be spliced into a URL.
fn is_scryfall_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => b.is_ascii_digit() || (b'a'..=b'f').contains(&b),
        })
}

/// The image CDN's URL for the FRONT of the card with Scryfall id `id` at
/// `version`: `https://cards.scryfall.io/<size>/front/<id[0]>/<id[1]>/<id>.jpg`.
///
/// This is where every `image_uris` entry Scryfall publishes points (less a
/// cache-busting query string), so it needs no API call. A double-faced card's
/// faces share the card's id — `front/` is its front face, the image the play
/// surface draws for the card's name — and a split, adventure or flip card is one
/// image. `None` when `id` is not shaped like a Scryfall id.
pub fn cdn_image_url(id: &str, version: Version) -> Option<String> {
    is_scryfall_id(id).then(|| {
        format!(
            "{IMAGE_CDN_PREFIX}{}/front/{}/{}/{id}.jpg",
            version.as_str(),
            &id[0..1],
            &id[1..2]
        )
    })
}

/// Pull the three CDN URLs out of a Scryfall card object.
///
/// A single-faced card (and a split card, which is one image) carries `image_uris`
/// at the top level. A double-faced card carries none there and one set per entry
/// of `card_faces`; the FRONT face (`card_faces[0]`) is the card's image in every
/// zone the play surface draws from a name. `None` when a size is absent or points
/// off the image CDN.
pub fn image_urls(card: &serde_json::Value) -> Option<[(Version, String); 3]> {
    let uris = match card.get("image_uris") {
        Some(uris) if uris.is_object() => uris,
        _ => card.get("card_faces")?.get(0)?.get("image_uris")?,
    };
    let url = |version: Version| -> Option<(Version, String)> {
        let text = uris.get(version.as_str())?.as_str()?;
        text.starts_with(IMAGE_CDN_PREFIX)
            .then(|| (version, text.to_string()))
    };
    Some([
        url(Version::Small)?,
        url(Version::Normal)?,
        url(Version::Large)?,
    ])
}

/// How long to wait after failed attempt number `attempt` (0-based) before the
/// next one: 2 s, 4 s, 8 s, 16 s, then capped at 30 s.
pub fn backoff_delay(attempt: u32) -> Duration {
    // `attempt.min(4)` first: a shift past the width would wrap to 0, not saturate.
    let secs = (2u64 << attempt.min(4)).min(30);
    Duration::from_secs(secs)
}

/// Why one HTTP GET failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// The server answered with this non-2xx status.
    Status(u16),
    /// DNS, TLS, timeout, a truncated body — anything short of a status.
    Transport(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::Status(code) => write!(f, "HTTP {code}"),
            FetchError::Transport(detail) => write!(f, "{detail}"),
        }
    }
}

/// The network, as far as this module is concerned: one GET, and the ability to
/// wait. Waiting is on the trait so a test can record the pauses instead of
/// sleeping through them.
pub trait Net {
    fn get(&mut self, url: &str, accept: &str) -> Result<Vec<u8>, FetchError>;
    fn pause(&mut self, duration: Duration);

    /// Called before each id-derived CDN download: make sure at least `gap` lies
    /// between the starts of consecutive such downloads. The default simply pauses
    /// for `gap` (always enough, and what a recording fake sees); [`HttpNet`]
    /// waits only for what is left of `gap` since the previous call.
    fn space(&mut self, gap: Duration) {
        self.pause(gap);
    }
}

/// Whether a GET of `url` must refuse redirects: every image download.
///
/// [`image_urls`] only accepts a URL on the image CDN, but that checks the FIRST
/// URL. `ureq` 3 follows up to 10 redirects by default (`Config::max_redirects`),
/// so without this a 3xx from the CDN could send the download to any host and the
/// bytes would still be stored. The one API call (`api.scryfall.com/cards/named`)
/// keeps `ureq`'s default: its answer is JSON that is parsed, never stored, and
/// the image URLs in it are host-checked again.
fn refuses_redirects(url: &str) -> bool {
    url.starts_with(IMAGE_CDN_PREFIX)
}

/// The `ureq` configuration both agents share: a global timeout and the
/// `User-Agent` Scryfall asks for.
fn base_config() -> ureq::config::ConfigBuilder<ureq::typestate::AgentScope> {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent(USER_AGENT)
}

/// The configuration image downloads use: https only and **no redirect is ever
/// followed**. With `max_redirects(0)` `ureq` hands a 3xx back as an ordinary
/// response instead of following it; [`HttpNet::get`] turns any non-2xx status
/// into [`FetchError::Status`], so a redirected image is a failed image.
fn cdn_config() -> ureq::config::Config {
    base_config().https_only(true).max_redirects(0).build()
}

/// The production [`Net`]: `ureq`, blocking, with a global timeout. **The only
/// networked code in this crate.** Constructed by `--prefetch-images` and by the
/// worker thread [`ImageService::with_prefetch`] starts; never by a test.
pub struct HttpNet {
    /// The Scryfall API call (JSON). `ureq`'s default redirect handling.
    api: ureq::Agent,
    /// Image downloads from the CDN. Never follows a redirect ([`cdn_config`]).
    cdn: ureq::Agent,
    /// When [`Net::space`] last returned.
    last_spaced: Option<std::time::Instant>,
}

impl HttpNet {
    pub fn new() -> Self {
        HttpNet {
            api: base_config().build().into(),
            cdn: cdn_config().into(),
            last_spaced: None,
        }
    }
}

impl Default for HttpNet {
    fn default() -> Self {
        Self::new()
    }
}

impl Net for HttpNet {
    fn get(&mut self, url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
        let agent = if refuses_redirects(url) {
            &self.cdn
        } else {
            &self.api
        };
        match agent.get(url).header("Accept", accept).call() {
            // `ureq` reports 4xx / 5xx as an error; a 3xx it was told not to
            // follow arrives here as a response. Only a 2xx carries the body
            // that was asked for.
            Ok(response) if !response.status().is_success() => {
                Err(FetchError::Status(response.status().as_u16()))
            }
            Ok(mut response) => response
                .body_mut()
                .read_to_vec()
                .map_err(|e| FetchError::Transport(e.to_string())),
            Err(ureq::Error::StatusCode(code)) => Err(FetchError::Status(code)),
            Err(other) => Err(FetchError::Transport(other.to_string())),
        }
    }

    fn pause(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }

    fn space(&mut self, gap: Duration) {
        if let Some(last) = self.last_spaced {
            std::thread::sleep(gap.saturating_sub(last.elapsed()));
        }
        self.last_spaced = Some(std::time::Instant::now());
    }
}

/// GET with bounded retries. A 429, a 5xx and a transport error are worth waiting
/// out ([`backoff_delay`]); any other status is an answer and is returned at once.
fn get_with_retry(net: &mut dyn Net, url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
    let mut last = FetchError::Transport("no attempt was made".to_string());
    for attempt in 0..MAX_ATTEMPTS {
        match net.get(url, accept) {
            Ok(bytes) => return Ok(bytes),
            Err(FetchError::Status(code)) if code != 429 && !(500..600).contains(&code) => {
                return Err(FetchError::Status(code));
            }
            Err(retryable) => last = retryable,
        }
        if attempt + 1 < MAX_ATTEMPTS {
            net.pause(backoff_delay(attempt));
        }
    }
    Err(last)
}

/// One polite call to the Scryfall API for `name`'s card object.
fn fetch_card_json(net: &mut dyn Net, name: &str) -> Result<Vec<u8>, FetchError> {
    net.pause(API_SPACING);
    get_with_retry(net, &named_card_url(name), "application/json")
}

/// The failure reason for a name Scryfall answered 404 to. The worker remembers
/// these (and only these) so a rematch does not ask again; a transient failure is
/// retried the next time the card is queued.
const UNKNOWN_NAME: &str = "Scryfall knows no card by this exact name";

/// What happened to one card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardOutcome {
    /// All three sizes were already in the store; nothing was requested.
    Present,
    /// This many images were downloaded and written.
    Fetched(usize),
    /// Nothing usable could be fetched; the card stays a text chip.
    Failed(String),
}

/// Which route a card's images took. Counted by the status route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    /// Nothing was requested: the store already held the card.
    Store,
    /// The image CDN only, from the Scryfall id. No `api.scryfall.com` request.
    CdnById,
    /// The `api.scryfall.com` named lookup was used (the fallback).
    NamedLookup,
}

/// Download one image and store it. `Err` is the failure reason.
fn download_into(
    net: &mut dyn Net,
    store: &ImageStore,
    name: &str,
    version: Version,
    url: &str,
) -> Result<(), DownloadError> {
    let bytes = get_with_retry(net, url, "image/jpeg").map_err(DownloadError::Fetch)?;
    // JPEG start-of-image marker. Anything else (an HTML error page, an empty
    // body) must not land in a file the route serves as `image/jpeg`.
    if !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Err(DownloadError::Other(format!(
            "{} image is not a JPEG",
            version.as_str()
        )));
    }
    store.write_atomic(name, version, &bytes).map_err(|e| {
        DownloadError::Other(format!(
            "{} image could not be stored: {e}",
            version.as_str()
        ))
    })
}

/// Why [`download_into`] failed: the GET itself, or what came back.
enum DownloadError {
    Fetch(FetchError),
    Other(String),
}

impl DownloadError {
    fn reason(self, version: Version) -> String {
        match self {
            DownloadError::Fetch(e) => format!("{} image: {e}", version.as_str()),
            DownloadError::Other(reason) => reason,
        }
    }
}

/// Make sure the store holds all three sizes of `name`, downloading what is
/// missing, and say which route was used. Never panics and never returns an
/// error — a failure is an outcome.
///
/// When `ids` knows the name, each missing size is one CDN download from the
/// id-derived URL ([`cdn_image_url`]) and `api.scryfall.com` is not contacted.
/// Otherwise — and once, if an id-derived URL answers 404 (an id the CDN no longer
/// serves) — the card goes through the named lookup: one API call for the card
/// object, then one CDN download per size still missing.
pub fn ensure_card_via(
    net: &mut dyn Net,
    store: &ImageStore,
    ids: &dyn CardIds,
    name: &str,
) -> (CardOutcome, Route) {
    let missing = missing_versions(store, name);
    if missing.is_empty() {
        return (CardOutcome::Present, Route::Store);
    }
    let mut written = 0;
    if let Some(id) = ids.scryfall_id(name) {
        let mut gone = false;
        for version in &missing {
            let Some(url) = cdn_image_url(&id, *version) else {
                // Not an id at all: the sqlite is not what this code expects.
                gone = true;
                break;
            };
            net.space(CDN_SPACING);
            match download_into(net, store, name, *version, &url) {
                Ok(()) => written += 1,
                Err(DownloadError::Fetch(FetchError::Status(404))) => {
                    gone = true;
                    break;
                }
                Err(e) => return (CardOutcome::Failed(e.reason(*version)), Route::CdnById),
            }
        }
        if !gone {
            return (CardOutcome::Fetched(written), Route::CdnById);
        }
    }
    let outcome = match ensure_by_named_lookup(net, store, name) {
        CardOutcome::Fetched(count) => CardOutcome::Fetched(written + count),
        other => other,
    };
    (outcome, Route::NamedLookup)
}

/// The fallback route: ask `api.scryfall.com` for the card object by exact name,
/// then download each size still missing from the URLs it carries.
fn ensure_by_named_lookup(net: &mut dyn Net, store: &ImageStore, name: &str) -> CardOutcome {
    let missing = missing_versions(store, name);
    if missing.is_empty() {
        return CardOutcome::Present;
    }
    let json = match fetch_card_json(net, name) {
        Ok(bytes) => bytes,
        // The engine names a double-faced card by its front face, which Scryfall's
        // exact match accepts. Should a `Front // Back` name ever arrive and be
        // refused, the front half alone is the same card.
        Err(FetchError::Status(404)) => match name.split_once(" // ") {
            Some((front, _)) => match fetch_card_json(net, front) {
                Ok(bytes) => bytes,
                Err(e) => return CardOutcome::Failed(format!("card lookup: {e}")),
            },
            None => return CardOutcome::Failed(UNKNOWN_NAME.to_string()),
        },
        Err(e) => return CardOutcome::Failed(format!("card lookup: {e}")),
    };
    let card: serde_json::Value = match serde_json::from_slice(&json) {
        Ok(card) => card,
        Err(e) => return CardOutcome::Failed(format!("card lookup returned non-JSON: {e}")),
    };
    let Some(urls) = image_urls(&card) else {
        return CardOutcome::Failed("the card object carries no usable image_uris".to_string());
    };
    let mut written = 0;
    for (version, url) in urls {
        if !missing.contains(&version) {
            continue;
        }
        if let Err(e) = download_into(net, store, name, version, &url) {
            return CardOutcome::Failed(e.reason(version));
        }
        written += 1;
    }
    CardOutcome::Fetched(written)
}

// ── Scryfall ids from `cards.sqlite` ──────────────────────────────────────────

/// Where a card name's Scryfall id comes from. One production implementation
/// ([`SqliteIds`]); [`NoIds`] when there is no sqlite; a map in tests.
pub trait CardIds {
    /// The Scryfall id of the card the play surface means by `name`, if known.
    fn scryfall_id(&self, name: &str) -> Option<String>;
}

/// No id source: every card takes the named lookup.
pub struct NoIds;

impl CardIds for NoIds {
    fn scryfall_id(&self, _name: &str) -> Option<String> {
        None
    }
}

// The two reads of `cards.sqlite`. This module deliberately does not depend on
// `card-db`'s schema module (it would pull the importer's types into the play
// server for a two-column read), so it names the columns itself:
// `cards(id, name, layout)` and `card_faces(card_id, face_index, name)`.
// HEADS-UP RULE (`memory/decisions.md`, 2026-09-14 entry): these columns have
// readers outside `card-db`; renaming one needs a heads-up to them — this file is
// one. A rename that arrives anyway is survived: `SqliteIds::open` fails its probe
// and the server falls back to the named lookup, loudly.
//
// The excluded layouts are Scryfall's non-game objects, which share names with
// real cards (same entry). `ORDER BY id` makes the pick among same-named rows
// (seventeen Un-set names with several variants) the same on every run.
const SQL_ID_BY_NAME: &str = "SELECT id FROM cards \
     WHERE name = ?1 \
       AND layout NOT IN ('art_series', 'token', 'double_faced_token', 'emblem', 'front_card') \
     ORDER BY id LIMIT 1";
// The engine names a double-faced (or adventure, or split) card by its front
// face, while `cards.name` is `Front // Back`: match face 0 and take its card.
const SQL_ID_BY_FRONT_FACE: &str = "SELECT c.id FROM card_faces f \
     JOIN cards c ON c.id = f.card_id \
     WHERE f.name = ?1 AND f.face_index = 0 \
       AND c.layout NOT IN ('art_series', 'token', 'double_faced_token', 'emblem', 'front_card') \
     ORDER BY c.id LIMIT 1";

/// Scryfall ids read from `cards.sqlite`, opened read-only.
pub struct SqliteIds {
    conn: rusqlite::Connection,
    /// Set once a query has failed, so the log line is written once.
    query_failed: std::cell::Cell<bool>,
}

impl SqliteIds {
    /// Open `path` read-only and check both queries compile against it. `Err`
    /// (the reason, for the log) when the file is missing, is not a database, or
    /// does not have the columns the queries name.
    pub fn open(path: &Path) -> Result<Self, String> {
        use rusqlite::OpenFlags;
        // Read-only and never created: a missing file is an error here, not an
        // empty database that would then "know" no names.
        let conn = rusqlite::Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|e| e.to_string())?;
        for sql in [SQL_ID_BY_NAME, SQL_ID_BY_FRONT_FACE] {
            conn.prepare_cached(sql).map_err(|e| e.to_string())?;
        }
        Ok(SqliteIds {
            conn,
            query_failed: std::cell::Cell::new(false),
        })
    }

    fn query(&self, sql: &str, name: &str) -> rusqlite::Result<Option<String>> {
        use rusqlite::OptionalExtension as _;
        self.conn
            .prepare_cached(sql)?
            .query_row([name], |row| row.get::<_, String>(0))
            .optional()
    }
}

impl CardIds for SqliteIds {
    fn scryfall_id(&self, name: &str) -> Option<String> {
        let found = self
            .query(SQL_ID_BY_NAME, name)
            .and_then(|by_name| match by_name {
                Some(id) => Ok(Some(id)),
                None => self.query(SQL_ID_BY_FRONT_FACE, name),
            });
        match found {
            Ok(id) => id,
            Err(e) => {
                if !self.query_failed.replace(true) {
                    eprintln!(
                        "card images: reading cards.sqlite failed ({e}); \
                         affected cards use the api.scryfall.com lookup"
                    );
                }
                None
            }
        }
    }
}

/// The id source the server and `--prefetch-images` use: `path` when it opens,
/// else [`NoIds`] with ONE loud line saying every card will take the rate-limited
/// lookup. The `String` is what the status route reports as `card_ids`.
pub fn open_card_ids(path: &Path) -> (Box<dyn CardIds + Send>, String) {
    match SqliteIds::open(path) {
        Ok(ids) => (Box::new(ids), path.display().to_string()),
        Err(reason) => {
            eprintln!(
                "card images: WARNING — no Scryfall ids: {} could not be used ({reason}). \
                 EVERY card will take the rate-limited api.scryfall.com lookup \
                 (minutes per deal, 429s likely). Point --cards-sqlite / \
                 SCUTEMOB_CARDS_SQLITE at the repo's cards.sqlite.",
                path.display()
            );
            (Box::new(NoIds), "none".to_string())
        }
    }
}

/// The default `cards.sqlite`: in the current directory when one is there (a
/// `cargo run` from the workspace root), else at the root of the workspace this
/// binary was built from.
pub fn default_cards_sqlite() -> PathBuf {
    let here = PathBuf::from("cards.sqlite");
    if here.is_file() {
        return here;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cards.sqlite")
}

// ── Decklists ─────────────────────────────────────────────────────────────────

/// Trim, drop empties, de-duplicate, sort.
fn distinct(names: impl IntoIterator<Item = String>) -> Vec<String> {
    names
        .into_iter()
        .map(|n| n.trim().to_string())
        .filter(|n| !n.is_empty())
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect()
}

/// Parse a decklist: one card per line, an optional leading count (`1 Sol Ring`,
/// `1x Sol Ring`), blank lines ignored, and a line that STARTS with `#` or `//` is
/// a comment (a `//` inside a line is part of a split card's name). Returns the
/// distinct names; counts are irrelevant to an image store.
pub fn parse_decklist(text: &str) -> Vec<String> {
    distinct(text.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            return None;
        }
        let name = match line.split_once(char::is_whitespace) {
            Some((first, rest)) if is_count(first) => rest,
            _ => line,
        };
        Some(name.to_string())
    }))
}

/// `3` or `3x` / `3X`.
fn is_count(token: &str) -> bool {
    let digits = token.strip_suffix(['x', 'X']).unwrap_or(token);
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

/// Parse an inline name list. Separated by `;` when the text contains one —
/// card names contain commas (`Atraxa, Praetors' Voice`) but never semicolons —
/// and by `,` otherwise.
pub fn parse_name_list(text: &str) -> Vec<String> {
    let separator = if text.contains(';') { ';' } else { ',' };
    distinct(text.split(separator).map(str::to_string))
}

/// Resolve `--prefetch-images <arg>`: a decklist file when `arg` names an existing
/// path, an inline name list otherwise. The only error is a path that exists and
/// cannot be read as text.
pub fn names_from_arg(arg: &str) -> std::io::Result<Vec<String>> {
    let path = Path::new(arg);
    if path.exists() {
        Ok(parse_decklist(&std::fs::read_to_string(path)?))
    } else {
        Ok(parse_name_list(arg))
    }
}

/// Every distinct card name a resolved table needs: each seat's commander and
/// main deck. Empty for a config whose decks are still a recipe
/// (`DeckSource::RandomPerSeat`) — `session::new_game` resolves it before this is
/// called.
pub fn deck_card_names(cfg: &LocalGameConfig) -> Vec<String> {
    let DeckSource::Fixed(decks) = &cfg.decks else {
        return Vec::new();
    };
    let names_by_id: HashMap<_, _> = mtg_engine::all_cards()
        .into_iter()
        .map(|def| (def.card_id, def.name))
        .collect();
    distinct(decks.iter().flat_map(|(_, deck)| {
        std::iter::once(&deck.commander)
            .chain(deck.main_deck.iter())
            .filter_map(|id| names_by_id.get(id).cloned())
    }))
}

// ── The service: store + optional background prefetch ─────────────────────────

/// Prefetch progress, as `GET /api/card-image/status` reports it. Counts are
/// cards (not files) and are cumulative for the life of the process.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PrefetchStatus {
    /// Whether this process fetches at all (`--no-image-prefetch` turns it off).
    pub enabled: bool,
    /// The store directory, as configured.
    pub store: String,
    /// Cards handed to the worker and not yet finished. `0` means the store is as
    /// ready as it is going to get.
    pub queued: usize,
    /// Cards for which at least one image was downloaded.
    pub fetched: usize,
    /// Cards that were already complete in the store when the worker reached them.
    pub present: usize,
    /// Names that could not be fetched. They render as text chips. **Serialized
    /// as their COUNT**: the names are cards of the table's decks (bot seats
    /// included), so the route's body must not carry them. The worker writes each
    /// name with its reason to stderr when it fails.
    #[serde(serialize_with = "serialize_len")]
    pub failed: Vec<String>,
    /// The subset of `failed` worth trying again (everything but a name Scryfall
    /// does not know). Each is retried when a later deal names it, and all of them
    /// by `POST /api/card-image/retry`. **Serialized as its COUNT**, like `failed`.
    #[serde(serialize_with = "serialize_len")]
    pub retry_queue: Vec<String>,
    /// Cards whose images came from the CDN alone, by Scryfall id — no
    /// `api.scryfall.com` request.
    pub by_id: usize,
    /// Cards that took the `api.scryfall.com` named lookup (the fallback): a name
    /// `cards.sqlite` does not hold, or an id path that answered 404.
    pub named_lookups: usize,
    /// The `cards.sqlite` the ids are read from, or `"none"` when it could not be
    /// opened (then every card is a named lookup).
    pub card_ids: String,
}

/// Serialize a list as its length (see [`PrefetchStatus::failed`]).
fn serialize_len<S: serde::Serializer>(list: &[String], serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_u64(list.len() as u64)
}

struct ServiceInner {
    store: ImageStore,
    status: Mutex<PrefetchStatus>,
    /// `Some` only when a worker thread exists. `Sender` is not `Sync`, hence the
    /// mutex; it is held for one non-blocking `send`.
    queue: Option<Mutex<mpsc::Sender<Vec<String>>>>,
}

/// The image half of the axum state: the store the route reads, and — only when
/// built with [`ImageService::with_prefetch`] — a background worker that fills it.
#[derive(Clone)]
pub struct ImageService {
    inner: Arc<ServiceInner>,
}

/// Lock that survives a panicked holder: the status is plain counters, and a
/// stale count is better than a dead status route.
fn lock_status(inner: &ServiceInner) -> std::sync::MutexGuard<'_, PrefetchStatus> {
    inner.status.lock().unwrap_or_else(|p| p.into_inner())
}

impl ImageService {
    /// Serve the store; never fetch. No thread is started and no network code is
    /// reachable from the value returned. This is what `AppState::new` builds, so
    /// a test cannot trigger a fetch by construction.
    pub fn serve_only(root: impl Into<PathBuf>) -> Self {
        Self::build(root.into(), None)
    }

    /// Serve the store and start the background worker that fills it, reading
    /// Scryfall ids from `cards_sqlite`. Called from the serving entry point only.
    pub fn with_prefetch(root: impl Into<PathBuf>, cards_sqlite: &Path) -> Self {
        let (ids, card_ids) = open_card_ids(cards_sqlite);
        Self::with_worker(root.into(), ids, card_ids, HttpNet::new)
    }

    /// [`ImageService::with_prefetch`] over any id source and any [`Net`]. The
    /// network value is built ON the worker thread (`make_net`), so it need not be
    /// `Send` itself.
    fn with_worker<N: Net>(
        root: PathBuf,
        ids: Box<dyn CardIds + Send>,
        card_ids: String,
        make_net: impl FnOnce() -> N + Send + 'static,
    ) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<String>>();
        let service = Self::build(root, Some(tx));
        lock_status(&service.inner).card_ids = card_ids;
        let inner = Arc::clone(&service.inner);
        let spawned = std::thread::Builder::new()
            .name("card-image-prefetch".to_string())
            .spawn(move || worker(&inner, &rx, &mut make_net(), ids.as_ref()));
        if let Err(e) = spawned {
            eprintln!(
                "card images: the prefetch worker could not start ({e}); serving the store only"
            );
        }
        service
    }

    fn build(root: PathBuf, tx: Option<mpsc::Sender<Vec<String>>>) -> Self {
        let status = PrefetchStatus {
            enabled: tx.is_some(),
            store: root.display().to_string(),
            card_ids: "none".to_string(),
            ..PrefetchStatus::default()
        };
        ImageService {
            inner: Arc::new(ServiceInner {
                store: ImageStore::new(root),
                status: Mutex::new(status),
                queue: tx.map(Mutex::new),
            }),
        }
    }

    pub fn store(&self) -> &ImageStore {
        &self.inner.store
    }

    pub fn prefetch_enabled(&self) -> bool {
        self.inner.queue.is_some()
    }

    pub fn status(&self) -> PrefetchStatus {
        lock_status(&self.inner).clone()
    }

    /// Hand card names to the background worker and return at once. Returns how
    /// many distinct names were queued — always `0` on a serve-only service.
    pub fn enqueue(&self, names: Vec<String>) -> usize {
        let Some(queue) = &self.inner.queue else {
            return 0;
        };
        let names = distinct(names);
        if names.is_empty() {
            return 0;
        }
        let count = names.len();
        lock_status(&self.inner).queued += count;
        let sent = queue
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .send(names)
            .is_ok();
        if !sent {
            // The worker is gone; nothing will ever drain these.
            let mut status = lock_status(&self.inner);
            status.queued = status.queued.saturating_sub(count);
            return 0;
        }
        count
    }

    /// Hand every card of the retry queue back to the worker. Returns how many
    /// were queued — `0` on a serve-only service or with nothing to retry. A name
    /// stays in the retry queue until the worker reaches it and it succeeds.
    pub fn retry_failed(&self) -> usize {
        let names = lock_status(&self.inner).retry_queue.clone();
        self.enqueue(names)
    }
}

/// Insert `name` into a sorted, duplicate-free list.
fn insert_sorted(list: &mut Vec<String>, name: &str) {
    if let Err(at) = list.binary_search_by(|n| n.as_str().cmp(name)) {
        list.insert(at, name.to_string());
    }
}

/// Fold one card's outcome into the status.
fn record(status: &mut PrefetchStatus, name: &str, outcome: &CardOutcome, route: Route) {
    status.queued = status.queued.saturating_sub(1);
    match route {
        Route::Store => {}
        Route::CdnById => status.by_id += 1,
        Route::NamedLookup => status.named_lookups += 1,
    }
    match outcome {
        CardOutcome::Present | CardOutcome::Fetched(_) => {
            if matches!(outcome, CardOutcome::Present) {
                status.present += 1;
            } else {
                status.fetched += 1;
            }
            status.failed.retain(|n| n != name);
            status.retry_queue.retain(|n| n != name);
        }
        CardOutcome::Failed(reason) => {
            insert_sorted(&mut status.failed, name);
            if reason == UNKNOWN_NAME {
                status.retry_queue.retain(|n| n != name);
            } else {
                insert_sorted(&mut status.retry_queue, name);
            }
        }
    }
}

/// The background worker: drains batches until every sender is dropped. Holds no
/// lock while it talks to the network, and never touches the game session.
fn worker(
    inner: &ServiceInner,
    rx: &mpsc::Receiver<Vec<String>>,
    net: &mut dyn Net,
    ids: &dyn CardIds,
) {
    // Names Scryfall has already refused in this process. A new game with the same
    // decks must not spend one API call per unknown name again.
    let mut refused: HashSet<String> = HashSet::new();
    while let Ok(batch) = rx.recv() {
        for name in batch {
            // A refused name asks the network nothing, so it took no route.
            let (outcome, route) = if refused.contains(&name) {
                (CardOutcome::Failed(UNKNOWN_NAME.to_string()), Route::Store)
            } else {
                ensure_card_via(net, &inner.store, ids, &name)
            };
            if let CardOutcome::Failed(reason) = &outcome {
                if !refused.contains(&name) {
                    eprintln!("card images: {name}: {reason}");
                }
                if reason == UNKNOWN_NAME {
                    refused.insert(name.clone());
                }
            }
            record(&mut lock_status(inner), &name, &outcome, route);
        }
    }
}

// ── `--prefetch-images` ───────────────────────────────────────────────────────

/// What a prefetch run did, by card.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct PrefetchSummary {
    pub fetched: Vec<String>,
    pub present: Vec<String>,
    pub failed: Vec<(String, String)>,
    /// How many cards took the `api.scryfall.com` named lookup (the fallback).
    pub named_lookups: usize,
}

/// Fetch every name into the store through `net`, reporting each card on `log`.
pub fn prefetch_all(
    net: &mut dyn Net,
    store: &ImageStore,
    ids: &dyn CardIds,
    names: &[String],
    log: &mut dyn FnMut(&str),
) -> PrefetchSummary {
    let mut summary = PrefetchSummary::default();
    for (index, name) in names.iter().enumerate() {
        let (outcome, route) = ensure_card_via(net, store, ids, name);
        if route == Route::NamedLookup {
            summary.named_lookups += 1;
        }
        let position = format!("[{}/{}]", index + 1, names.len());
        match outcome {
            CardOutcome::Present => {
                log(&format!("{position} present  {name}"));
                summary.present.push(name.clone());
            }
            CardOutcome::Fetched(count) => {
                log(&format!("{position} fetched  {name} ({count} images)"));
                summary.fetched.push(name.clone());
            }
            CardOutcome::Failed(reason) => {
                log(&format!("{position} FAILED   {name}: {reason}"));
                summary.failed.push((name.clone(), reason));
            }
        }
    }
    summary
}

/// `play-server --prefetch-images <decklist-file | names>`: fill the store and
/// return. `Err` only when the argument could not be read; a card Scryfall does
/// not know is a reported failure in the summary, not an error.
pub fn run_cli_prefetch(store_root: &Path, cards_sqlite: &Path, arg: &str) -> anyhow::Result<()> {
    use anyhow::Context as _;

    let names =
        names_from_arg(arg).with_context(|| format!("--prefetch-images: could not read {arg}"))?;
    let store = ImageStore::new(store_root);
    println!(
        "Prefetching {} card(s) x 3 sizes into {}",
        names.len(),
        store.root().display()
    );
    let (ids, _) = open_card_ids(cards_sqlite);
    let mut net = HttpNet::new();
    let summary = prefetch_all(&mut net, &store, ids.as_ref(), &names, &mut |line| {
        println!("{line}")
    });
    println!(
        "Done: {} fetched, {} already present, {} failed; {} took the api.scryfall.com lookup.",
        summary.fetched.len(),
        summary.present.len(),
        summary.failed.len(),
        summary.named_lookups
    );
    for (name, reason) in &summary.failed {
        println!("  failed: {name} — {reason}");
    }
    Ok(())
}

/// The default store directory, resolved the way `dist/` is: next to the
/// executable when an `images/` already sits there (an installed build), else
/// `tools/play-server/images` under the current directory when that crate
/// directory is there (a `cargo run` from the workspace root), else the crate
/// directory this binary was built from.
pub fn default_store_dir() -> PathBuf {
    let beside_exe = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|pp| pp.join("images")));
    if let Some(dir) = beside_exe.filter(|d| d.is_dir()) {
        return dir;
    }
    let crate_dir = PathBuf::from("tools/play-server");
    if crate_dir.is_dir() {
        return crate_dir.join("images");
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("images")
}

// ── Routes ────────────────────────────────────────────────────────────────────

/// A served image may be cached for 30 days: a card's image at a size does not
/// change, and the URL carries both.
const CACHE_HIT: &str = "public, max-age=2592000";
/// A miss must not be cached, so an image prefetched a minute later shows up on
/// the next request without a hard reload.
const CACHE_MISS: &str = "no-store";

fn failure(status: StatusCode, kind: &'static str, error: impl Into<String>) -> Response {
    let mut response = ApiFailure {
        status,
        body: ApiError {
            error: error.into(),
            kind,
        },
    }
    .into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_MISS));
    response
}

/// `GET /api/card-image?name=<exact card name>&version=<small|normal|large>`.
///
/// | Status | When |
/// |---|---|
/// | 200 `image/jpeg` | the store holds this card at this size |
/// | 400 `bad_params` | `name` missing or empty, `version` missing or not one of the three |
/// | 404 `image_not_in_store` | not in the store — the client shows its chip |
///
/// Reads one file. Never fetches, never reads the session (module doc).
pub async fn get_card_image(
    State(state): State<SharedState>,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let name = query.get("name").map(String::as_str).unwrap_or("");
    if name.trim().is_empty() {
        return failure(
            StatusCode::BAD_REQUEST,
            "bad_params",
            "`name` is required: the exact card name",
        );
    }
    let Some(version) = query.get("version").and_then(|v| Version::parse(v)) else {
        return failure(
            StatusCode::BAD_REQUEST,
            "bad_params",
            "`version` is required and must be one of small, normal, large",
        );
    };
    let path = state.images.store().path_for(name, version);
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg")),
                (header::CACHE_CONTROL, HeaderValue::from_static(CACHE_HIT)),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => failure(
            StatusCode::NOT_FOUND,
            "image_not_in_store",
            format!(
                "no {} image for {name:?} in the local store; prefetch it (see the README)",
                version.as_str()
            ),
        ),
    }
}

/// `GET /api/card-image/status` — prefetch progress. `queued == 0` means done.
/// Every field but `store` and `enabled` is a count; `failed` is a number, never
/// the names (module doc, "Hidden information").
pub async fn get_card_image_status(State(state): State<SharedState>) -> Response {
    let mut response = Json(state.images.status()).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_MISS));
    response
}

/// `POST /api/card-image/retry` — hand every card of the retry queue back to the
/// prefetch worker now, and answer with the status (same body as the status
/// route, plus `requeued`: how many cards this call queued). Counts only. Reads no
/// session and fetches nothing itself; on a serve-only server `requeued` is `0`.
pub async fn post_card_image_retry(State(state): State<SharedState>) -> Response {
    let requeued = state.images.retry_failed();
    let mut body = serde_json::to_value(state.images.status()).unwrap_or_default();
    if let Some(object) = body.as_object_mut() {
        object.insert("requeued".to_string(), requeued.into());
    }
    let mut response = Json(body).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static(CACHE_MISS));
    response
}

// ── Tests ─────────────────────────────────────────────────────────────────────
//
// Nothing below opens a socket or constructs the production network type: the
// fetch procedure is driven through a scripted fake, and the routes through the
// router with `oneshot` against a store directory under the system temp dir.

#[cfg(test)]
mod tests {
    use super::*;

    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicU32, Ordering};

    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use mtg_simulator::BotKind;
    use tower::ServiceExt;

    use crate::session::{self, AppState, NewGameDefaults};

    const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, b'J', b'F', b'I', b'F'];

    /// A fresh, uniquely named store directory under the system temp dir, removed
    /// when the guard drops.
    struct TempStore {
        root: PathBuf,
    }

    impl TempStore {
        fn new(tag: &str) -> Self {
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let root = std::env::temp_dir().join(format!(
                "scutemob-images-test-{}-{}-{tag}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = std::fs::remove_dir_all(&root);
            TempStore { root }
        }

        fn store(&self) -> ImageStore {
            ImageStore::new(&self.root)
        }
    }

    impl Drop for TempStore {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    /// A scripted network: answers are popped per URL, every call and every pause
    /// is recorded, and an unscripted URL is a 404.
    #[derive(Default)]
    struct FakeNet {
        answers: HashMap<String, VecDeque<Result<Vec<u8>, FetchError>>>,
        calls: Vec<(String, String)>,
        pauses: Vec<Duration>,
    }

    impl FakeNet {
        fn script(&mut self, url: &str, answer: Result<Vec<u8>, FetchError>) {
            self.answers
                .entry(url.to_string())
                .or_default()
                .push_back(answer);
        }

        fn calls_to(&self, host_prefix: &str) -> usize {
            self.calls
                .iter()
                .filter(|(url, _)| url.starts_with(host_prefix))
                .count()
        }
    }

    impl Net for FakeNet {
        fn get(&mut self, url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
            self.calls.push((url.to_string(), accept.to_string()));
            self.answers
                .get_mut(url)
                .and_then(VecDeque::pop_front)
                .unwrap_or(Err(FetchError::Status(404)))
        }

        fn pause(&mut self, duration: Duration) {
            self.pauses.push(duration);
        }
    }

    /// Hand-written: the fields of a Scryfall card object this module reads, for
    /// a single-faced card.
    const SOL_RING_JSON: &str = r#"{
        "object": "card",
        "name": "Sol Ring",
        "layout": "normal",
        "image_uris": {
            "small": "https://cards.scryfall.io/small/front/a/b/sol.jpg?1",
            "normal": "https://cards.scryfall.io/normal/front/a/b/sol.jpg?1",
            "large": "https://cards.scryfall.io/large/front/a/b/sol.jpg?1",
            "png": "https://cards.scryfall.io/png/front/a/b/sol.png?1"
        }
    }"#;

    /// Hand-written: a double-faced card has no top-level `image_uris`; each face
    /// carries its own.
    const DFC_JSON: &str = r#"{
        "object": "card",
        "name": "Bloodline Keeper // Lord of Lineage",
        "layout": "transform",
        "card_faces": [
            { "name": "Bloodline Keeper", "image_uris": {
                "small": "https://cards.scryfall.io/small/front/c/d/keeper.jpg?2",
                "normal": "https://cards.scryfall.io/normal/front/c/d/keeper.jpg?2",
                "large": "https://cards.scryfall.io/large/front/c/d/keeper.jpg?2" } },
            { "name": "Lord of Lineage", "image_uris": {
                "small": "https://cards.scryfall.io/small/back/c/d/keeper.jpg?2",
                "normal": "https://cards.scryfall.io/normal/back/c/d/keeper.jpg?2",
                "large": "https://cards.scryfall.io/large/back/c/d/keeper.jpg?2" } }
        ]
    }"#;

    /// The fetch procedure with NO id source: every missing card takes the named
    /// lookup. The tests that pin that route (it predates the id route) call this.
    fn ensure_card(net: &mut dyn Net, store: &ImageStore, name: &str) -> CardOutcome {
        let (outcome, route) = ensure_card_via(net, store, &NoIds, name);
        assert_eq!(
            route == Route::Store,
            outcome == CardOutcome::Present,
            "only a card already in the store takes no route"
        );
        outcome
    }

    /// An id source that is a map, and a real Scryfall-shaped id to put in it.
    impl CardIds for HashMap<&'static str, &'static str> {
        fn scryfall_id(&self, name: &str) -> Option<String> {
            self.get(name).map(|id| id.to_string())
        }
    }

    const SOL_RING_ID: &str = "8ee443cc-e17a-493b-9c93-1f9e141a30e4";

    fn sol_ring_ids() -> HashMap<&'static str, &'static str> {
        HashMap::from([("Sol Ring", SOL_RING_ID)])
    }

    fn id_url(id: &str, version: Version) -> String {
        cdn_image_url(id, version).expect("a well-formed id")
    }

    fn sol_ring_net() -> FakeNet {
        let mut net = FakeNet::default();
        net.script(
            &named_card_url("Sol Ring"),
            Ok(SOL_RING_JSON.as_bytes().to_vec()),
        );
        for size in ["small", "normal", "large"] {
            net.script(
                &format!("https://cards.scryfall.io/{size}/front/a/b/sol.jpg?1"),
                Ok(JPEG.to_vec()),
            );
        }
        net
    }

    // ── slug ──

    #[test]
    fn test_image_slug_alphabet_cannot_escape_the_store_root() {
        let hostile = [
            "../../etc/passwd",
            "/etc/passwd",
            "..",
            ".",
            "",
            "a/../../b",
            "C:\\Windows\\system32",
            "name\0with-nul",
            "Fire // Ice",
            "Lim-Dûl's Vault",
            "Atraxa, Praetors' Voice",
            "%2e%2e%2f",
        ];
        let store = ImageStore::new("/store-root");
        for name in hostile {
            let s = slug(name);
            assert!(
                !s.is_empty()
                    && s.bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-'),
                "slug({name:?}) = {s:?} leaves [a-z0-9-]"
            );
            for version in Version::ALL {
                let path = store.path_for(name, version);
                assert_eq!(
                    path.parent(),
                    Some(Path::new("/store-root").join(version.as_str()).as_path()),
                    "{name:?} must land directly in the version directory, got {path:?}"
                );
                assert!(path
                    .components()
                    .all(|c| c != std::path::Component::ParentDir));
            }
        }
    }

    #[test]
    fn test_image_slug_is_readable_stable_and_bounded() {
        let s = slug("Atraxa, Praetors' Voice");
        assert!(s.starts_with("atraxa-praetors-voice-"), "{s}");
        assert_eq!(s.len(), "atraxa-praetors-voice-".len() + 16);
        // Pinned: the file name on disk must not move between builds.
        assert_eq!(
            slug("Sol Ring"),
            format!("sol-ring-{:016x}", fnv1a64("Sol Ring"))
        );
        assert_eq!(fnv1a64(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a64("a"), 0xaf63_dc4c_8601_ec8c);
        assert!(slug(&"x".repeat(500)).len() <= SLUG_STEM_MAX + 17);
        assert!(slug("!!!").starts_with("card-"));
    }

    #[test]
    fn test_image_slug_names_differing_only_in_punctuation_do_not_collide() {
        let groups: [&[&str]; 3] = [
            &[
                "Fire // Ice",
                "Fire Ice",
                "Fire-Ice",
                "fire ice",
                "Fire, Ice",
            ],
            &["Lim-Dûl's Vault", "Lim-Dul's Vault", "Lim-D l's Vault"],
            &["Sol Ring", "Sol Ring ", " Sol Ring", "Sol  Ring"],
        ];
        for group in groups {
            let slugs: BTreeSet<String> = group.iter().map(|n| slug(n)).collect();
            assert_eq!(
                slugs.len(),
                group.len(),
                "collision inside {group:?}: {slugs:?}"
            );
        }
    }

    #[test]
    fn test_image_version_parses_exactly_three_spellings() {
        for version in Version::ALL {
            assert_eq!(Version::parse(version.as_str()), Some(version));
        }
        for bad in [
            "",
            "Small",
            "png",
            "art_crop",
            "border_crop",
            "small ",
            "../small",
        ] {
            assert_eq!(Version::parse(bad), None, "{bad:?}");
        }
    }

    // ── planner ──

    #[test]
    fn test_image_plan_is_versions_minus_what_the_store_holds() {
        let temp = TempStore::new("plan");
        let store = temp.store();
        assert_eq!(missing_versions(&store, "Sol Ring"), Version::ALL.to_vec());
        store
            .write_atomic("Sol Ring", Version::Normal, JPEG)
            .unwrap();
        assert_eq!(
            missing_versions(&store, "Sol Ring"),
            vec![Version::Small, Version::Large]
        );
        // Another card's files do not count.
        assert_eq!(
            missing_versions(&store, "Sol Ring 2"),
            Version::ALL.to_vec()
        );
        store
            .write_atomic("Sol Ring", Version::Small, JPEG)
            .unwrap();
        store
            .write_atomic("Sol Ring", Version::Large, JPEG)
            .unwrap();
        assert!(missing_versions(&store, "Sol Ring").is_empty());
        // The atomic write leaves no temp file behind.
        let leftovers: Vec<_> = std::fs::read_dir(temp.root.join("small"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_none_or(|x| x != "jpg"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }

    // ── Scryfall JSON, URLs, backoff ──

    #[test]
    fn test_image_urls_come_from_the_top_level_for_a_single_faced_card() {
        let card: serde_json::Value = serde_json::from_str(SOL_RING_JSON).unwrap();
        let urls = image_urls(&card).expect("three urls");
        assert_eq!(
            urls[0],
            (
                Version::Small,
                "https://cards.scryfall.io/small/front/a/b/sol.jpg?1".to_string()
            )
        );
        assert_eq!(urls[1].0, Version::Normal);
        assert_eq!(
            urls[2],
            (
                Version::Large,
                "https://cards.scryfall.io/large/front/a/b/sol.jpg?1".to_string()
            )
        );
    }

    #[test]
    fn test_image_urls_come_from_the_front_face_of_a_double_faced_card() {
        let card: serde_json::Value = serde_json::from_str(DFC_JSON).unwrap();
        let urls = image_urls(&card).expect("three urls");
        for (version, url) in urls {
            assert_eq!(
                url,
                format!(
                    "https://cards.scryfall.io/{}/front/c/d/keeper.jpg?2",
                    version.as_str()
                )
            );
        }
    }

    #[test]
    fn test_image_urls_refuse_missing_sizes_and_foreign_hosts() {
        let no_images = serde_json::json!({ "object": "card", "name": "X" });
        assert!(image_urls(&no_images).is_none());
        let error = serde_json::json!({ "object": "error", "status": 404 });
        assert!(image_urls(&error).is_none());
        let partial = serde_json::json!({ "image_uris": {
            "small": "https://cards.scryfall.io/s.jpg", "normal": "https://cards.scryfall.io/n.jpg" } });
        assert!(image_urls(&partial).is_none());
        let foreign = serde_json::json!({ "image_uris": {
            "small": "https://cards.scryfall.io/s.jpg",
            "normal": "https://example.com/n.jpg",
            "large": "https://cards.scryfall.io/l.jpg" } });
        assert!(image_urls(&foreign).is_none());
    }

    #[test]
    fn test_image_named_card_url_percent_encodes_the_exact_name() {
        assert_eq!(
            named_card_url("Sol Ring"),
            "https://api.scryfall.com/cards/named?exact=Sol%20Ring"
        );
        assert_eq!(
            named_card_url("Atraxa, Praetors' Voice"),
            "https://api.scryfall.com/cards/named?exact=Atraxa%2C%20Praetors%27%20Voice"
        );
        assert_eq!(
            named_card_url("Fire // Ice&format=image"),
            "https://api.scryfall.com/cards/named?exact=Fire%20%2F%2F%20Ice%26format%3Dimage"
        );
        assert!(named_card_url("Lim-Dûl's Vault").ends_with("Lim-D%C3%BBl%27s%20Vault"));
    }

    #[test]
    fn test_image_backoff_schedule_grows_and_is_capped() {
        let secs: Vec<u64> = (0..6).map(|a| backoff_delay(a).as_secs()).collect();
        assert_eq!(secs, vec![2, 4, 8, 16, 30, 30]);
        assert_eq!(backoff_delay(63).as_secs(), 30);
        assert_eq!(backoff_delay(64).as_secs(), 30);
        assert_eq!(backoff_delay(u32::MAX).as_secs(), 30);
        // Scryfall's stated floor between API calls is 50-100 ms.
        assert!(API_SPACING >= Duration::from_millis(100));
    }

    // ── the fetch procedure, against the scripted fake ──

    #[test]
    fn test_image_ensure_card_makes_one_api_call_then_three_cdn_downloads() {
        let temp = TempStore::new("ensure");
        let store = temp.store();
        let mut net = sol_ring_net();
        assert_eq!(
            ensure_card(&mut net, &store, "Sol Ring"),
            CardOutcome::Fetched(3)
        );
        assert_eq!(net.calls_to("https://api.scryfall.com/"), 1);
        assert_eq!(net.calls_to("https://cards.scryfall.io/"), 3);
        assert_eq!(net.calls[0].1, "application/json");
        assert!(net.calls[1..]
            .iter()
            .all(|(_, accept)| accept == "image/jpeg"));
        assert_eq!(
            net.pauses,
            vec![API_SPACING],
            "one polite pause, before the API call"
        );
        for version in Version::ALL {
            assert_eq!(
                std::fs::read(store.path_for("Sol Ring", version)).unwrap(),
                JPEG
            );
        }
        // A second pass asks the network nothing at all.
        let mut silent = FakeNet::default();
        assert_eq!(
            ensure_card(&mut silent, &store, "Sol Ring"),
            CardOutcome::Present
        );
        assert!(silent.calls.is_empty() && silent.pauses.is_empty());
    }

    #[test]
    fn test_image_ensure_card_downloads_only_the_missing_sizes() {
        let temp = TempStore::new("partial");
        let store = temp.store();
        store
            .write_atomic("Sol Ring", Version::Small, b"\xFF\xD8\xFFold")
            .unwrap();
        let mut net = sol_ring_net();
        assert_eq!(
            ensure_card(&mut net, &store, "Sol Ring"),
            CardOutcome::Fetched(2)
        );
        assert_eq!(net.calls_to("https://cards.scryfall.io/small/"), 0);
        assert_eq!(
            std::fs::read(store.path_for("Sol Ring", Version::Small)).unwrap(),
            b"\xFF\xD8\xFFold",
            "a present file is never rewritten"
        );
    }

    #[test]
    fn test_image_ensure_card_retries_a_429_with_growing_backoff() {
        let temp = TempStore::new("retry");
        let store = temp.store();
        let mut net = FakeNet::default();
        let api = named_card_url("Sol Ring");
        net.script(&api, Err(FetchError::Status(429)));
        net.script(&api, Err(FetchError::Transport("timed out".to_string())));
        net.script(&api, Err(FetchError::Status(503)));
        net.script(&api, Ok(SOL_RING_JSON.as_bytes().to_vec()));
        for size in ["small", "normal", "large"] {
            net.script(
                &format!("https://cards.scryfall.io/{size}/front/a/b/sol.jpg?1"),
                Ok(JPEG.to_vec()),
            );
        }
        assert_eq!(
            ensure_card(&mut net, &store, "Sol Ring"),
            CardOutcome::Fetched(3)
        );
        assert_eq!(net.calls_to("https://api.scryfall.com/"), 4);
        assert_eq!(
            net.pauses,
            vec![
                API_SPACING,
                backoff_delay(0),
                backoff_delay(1),
                backoff_delay(2)
            ]
        );
    }

    #[test]
    fn test_image_ensure_card_gives_up_after_a_bounded_number_of_attempts() {
        let temp = TempStore::new("bounded");
        let store = temp.store();
        let mut net = FakeNet::default();
        let api = named_card_url("Sol Ring");
        for _ in 0..50 {
            net.script(&api, Err(FetchError::Status(429)));
        }
        let outcome = ensure_card(&mut net, &store, "Sol Ring");
        assert_eq!(
            outcome,
            CardOutcome::Failed("card lookup: HTTP 429".to_string())
        );
        assert_eq!(net.calls.len(), MAX_ATTEMPTS as usize);
        // No pause after the last attempt: there is nothing left to wait for.
        assert_eq!(net.pauses.len(), MAX_ATTEMPTS as usize);
        assert!(missing_versions(&store, "Sol Ring").len() == 3);
    }

    #[test]
    fn test_image_ensure_card_records_an_unknown_name_as_failed_without_retrying() {
        let temp = TempStore::new("unknown");
        let store = temp.store();
        let mut net = FakeNet::default();
        let outcome = ensure_card(&mut net, &store, "Goblin Token");
        assert!(matches!(outcome, CardOutcome::Failed(_)), "{outcome:?}");
        assert_eq!(
            net.calls.len(),
            1,
            "a 404 is an answer, not a reason to retry"
        );
        assert!(!temp.root.exists(), "a failure writes nothing");
    }

    #[test]
    fn test_image_ensure_card_falls_back_to_the_front_half_of_a_two_part_name() {
        let temp = TempStore::new("front");
        let store = temp.store();
        let mut net = FakeNet::default();
        let name = "Bloodline Keeper // Lord of Lineage";
        net.script(
            &named_card_url("Bloodline Keeper"),
            Ok(DFC_JSON.as_bytes().to_vec()),
        );
        for size in ["small", "normal", "large"] {
            net.script(
                &format!("https://cards.scryfall.io/{size}/front/c/d/keeper.jpg?2"),
                Ok(JPEG.to_vec()),
            );
        }
        assert_eq!(ensure_card(&mut net, &store, name), CardOutcome::Fetched(3));
        assert!(
            store.has(name, Version::Large),
            "stored under the name that was asked for"
        );
        assert_eq!(net.calls_to("https://cards.scryfall.io/small/back/"), 0);
    }

    // ── CDN by Scryfall id (`scutemob-270`) ──

    #[test]
    fn test_image_cdn_url_is_derived_from_the_scryfall_id() {
        assert_eq!(
            cdn_image_url(SOL_RING_ID, Version::Small).as_deref(),
            Some(
                "https://cards.scryfall.io/small/front/8/e/8ee443cc-e17a-493b-9c93-1f9e141a30e4.jpg"
            )
        );
        for version in Version::ALL {
            let url = id_url(SOL_RING_ID, version);
            assert!(url.starts_with(&format!(
                "https://cards.scryfall.io/{}/front/8/e/",
                version.as_str()
            )));
            assert!(
                refuses_redirects(&url),
                "an id-derived URL is fetched by the redirect-refusing agent"
            );
        }
    }

    #[test]
    fn test_image_cdn_url_refuses_anything_not_shaped_like_a_scryfall_id() {
        for bad in [
            "",
            "sol-ring",
            "8ee443cc-e17a-493b-9c93-1f9e141a30e",
            "8ee443cc-e17a-493b-9c93-1f9e141a30e44",
            "8EE443CC-E17A-493B-9C93-1F9E141A30E4",
            "8ee443cce17a493b9c931f9e141a30e4",
            "../../../../etc/passwd/aaaaaaaaaaaaa",
            "8ee443cc-e17a-493b-9c93-1f9e141a30e/",
            "8ee443cc/e17a-493b-9c93-1f9e141a30e4",
            "8ee443cc-e17a-493b-9c93-1f9e141a30e?",
            "ééééééééééééééééé-",
        ] {
            assert_eq!(cdn_image_url(bad, Version::Small), None, "{bad:?}");
        }
    }

    #[test]
    fn test_image_ensure_card_by_id_makes_three_cdn_downloads_and_no_api_call() {
        let temp = TempStore::new("by-id");
        let store = temp.store();
        let mut net = FakeNet::default();
        for version in Version::ALL {
            net.script(&id_url(SOL_RING_ID, version), Ok(JPEG.to_vec()));
        }
        assert_eq!(
            ensure_card_via(&mut net, &store, &sol_ring_ids(), "Sol Ring"),
            (CardOutcome::Fetched(3), Route::CdnById)
        );
        assert_eq!(
            net.calls_to("https://api.scryfall.com/"),
            0,
            "a known id needs no api.scryfall.com request"
        );
        assert_eq!(net.calls.len(), 3);
        assert!(net.calls.iter().all(|(url, accept)| {
            url.starts_with("https://cards.scryfall.io/") && accept == "image/jpeg"
        }));
        assert_eq!(
            net.pauses,
            vec![CDN_SPACING; 3],
            "spaced before every CDN download"
        );
        assert!(CDN_SPACING >= Duration::from_millis(100));
        for version in Version::ALL {
            assert_eq!(
                std::fs::read(store.path_for("Sol Ring", version)).unwrap(),
                JPEG
            );
        }
        // Present now: nothing is asked of the network or of the id source's route.
        let mut silent = FakeNet::default();
        assert_eq!(
            ensure_card_via(&mut silent, &store, &sol_ring_ids(), "Sol Ring"),
            (CardOutcome::Present, Route::Store)
        );
        assert!(silent.calls.is_empty() && silent.pauses.is_empty());
    }

    #[test]
    fn test_image_ensure_card_by_id_downloads_only_the_missing_sizes() {
        let temp = TempStore::new("by-id-partial");
        let store = temp.store();
        store
            .write_atomic("Sol Ring", Version::Normal, JPEG)
            .unwrap();
        let mut net = FakeNet::default();
        for version in Version::ALL {
            net.script(&id_url(SOL_RING_ID, version), Ok(JPEG.to_vec()));
        }
        assert_eq!(
            ensure_card_via(&mut net, &store, &sol_ring_ids(), "Sol Ring"),
            (CardOutcome::Fetched(2), Route::CdnById)
        );
        assert_eq!(net.calls_to("https://cards.scryfall.io/normal/"), 0);
        assert_eq!(net.calls.len(), 2);
    }

    #[test]
    fn test_image_ensure_card_id_path_404_falls_back_to_the_named_lookup_exactly_once() {
        let temp = TempStore::new("by-id-404");
        let store = temp.store();
        // Every id-derived URL is unscripted, so each would answer 404. The named
        // lookup and the URLs its JSON carries are scripted.
        let mut net = sol_ring_net();
        assert_eq!(
            ensure_card_via(&mut net, &store, &sol_ring_ids(), "Sol Ring"),
            (CardOutcome::Fetched(3), Route::NamedLookup)
        );
        assert_eq!(
            net.calls_to("https://api.scryfall.com/"),
            1,
            "the named lookup, exactly once for the card"
        );
        assert_eq!(
            net.calls[0].0,
            id_url(SOL_RING_ID, Version::Small),
            "the id path is tried first"
        );
        assert_eq!(
            net.calls
                .iter()
                .filter(|(url, _)| url.contains(SOL_RING_ID))
                .count(),
            1,
            "one 404 settles it: the other sizes are not asked for by id"
        );
        assert_eq!(net.calls.len(), 5, "1 id miss + 1 lookup + 3 downloads");
        assert_eq!(net.pauses, vec![CDN_SPACING, API_SPACING], "no backoff");
        assert!(missing_versions(&store, "Sol Ring").is_empty());
    }

    #[test]
    fn test_image_ensure_card_id_path_404_mid_card_keeps_what_was_stored() {
        let temp = TempStore::new("by-id-404-mid");
        let store = temp.store();
        let mut net = sol_ring_net();
        net.script(&id_url(SOL_RING_ID, Version::Small), Ok(JPEG.to_vec()));
        assert_eq!(
            ensure_card_via(&mut net, &store, &sol_ring_ids(), "Sol Ring"),
            (CardOutcome::Fetched(3), Route::NamedLookup)
        );
        assert_eq!(net.calls_to("https://api.scryfall.com/"), 1);
        assert_eq!(
            net.calls_to("https://cards.scryfall.io/small/"),
            1,
            "the size the id path already stored is not downloaded again"
        );
    }

    #[test]
    fn test_image_ensure_card_name_absent_from_the_ids_takes_the_named_lookup() {
        let temp = TempStore::new("absent");
        let store = temp.store();
        let mut net = sol_ring_net();
        let ids: HashMap<&'static str, &'static str> = HashMap::from([("Forest", SOL_RING_ID)]);
        assert_eq!(
            ensure_card_via(&mut net, &store, &ids, "Sol Ring"),
            (CardOutcome::Fetched(3), Route::NamedLookup)
        );
        assert_eq!(net.calls_to("https://api.scryfall.com/"), 1);
        assert_eq!(net.pauses, vec![API_SPACING]);
        // And a name neither knows is a failure that route owns.
        let mut net = FakeNet::default();
        let (outcome, route) = ensure_card_via(&mut net, &store, &ids, "Steel Guardian");
        assert_eq!(outcome, CardOutcome::Failed(UNKNOWN_NAME.to_string()));
        assert_eq!(route, Route::NamedLookup);
    }

    #[test]
    fn test_image_ensure_card_by_id_retries_a_429_and_fails_without_an_api_call() {
        let temp = TempStore::new("by-id-429");
        let store = temp.store();
        let mut net = FakeNet::default();
        let small = id_url(SOL_RING_ID, Version::Small);
        for _ in 0..MAX_ATTEMPTS {
            net.script(&small, Err(FetchError::Status(429)));
        }
        assert_eq!(
            ensure_card_via(&mut net, &store, &sol_ring_ids(), "Sol Ring"),
            (
                CardOutcome::Failed("small image: HTTP 429".to_string()),
                Route::CdnById
            )
        );
        assert_eq!(net.calls.len(), MAX_ATTEMPTS as usize);
        assert_eq!(
            net.calls_to("https://api.scryfall.com/"),
            0,
            "a rate-limited CDN is not a reason to go to the rate-limited API"
        );
        assert_eq!(net.pauses[0], CDN_SPACING);
        assert_eq!(
            net.pauses[1..],
            [
                backoff_delay(0),
                backoff_delay(1),
                backoff_delay(2),
                backoff_delay(3)
            ]
        );
    }

    #[test]
    fn test_image_ensure_card_a_malformed_id_takes_the_named_lookup() {
        let temp = TempStore::new("bad-id");
        let store = temp.store();
        let mut net = sol_ring_net();
        let ids: HashMap<&'static str, &'static str> =
            HashMap::from([("Sol Ring", "../../not-an-id")]);
        assert_eq!(
            ensure_card_via(&mut net, &store, &ids, "Sol Ring"),
            (CardOutcome::Fetched(3), Route::NamedLookup)
        );
        assert!(
            net.calls.iter().all(|(url, _)| !url.contains("not-an-id")),
            "a malformed id never reaches a URL"
        );
    }

    // ── cards.sqlite ──

    /// A sqlite file with the columns this module reads, in a temp directory.
    fn sqlite_fixture(temp: &TempStore, schema: &str) -> PathBuf {
        std::fs::create_dir_all(&temp.root).unwrap();
        let path = temp.root.join("cards.sqlite");
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(schema).unwrap();
        path
    }

    const FIXTURE_SCHEMA: &str = "
        CREATE TABLE cards (id TEXT PRIMARY KEY, name TEXT NOT NULL, layout TEXT NOT NULL);
        CREATE TABLE card_faces (card_id TEXT NOT NULL, face_index INTEGER NOT NULL, name TEXT NOT NULL);
        INSERT INTO cards VALUES
          ('8ee443cc-e17a-493b-9c93-1f9e141a30e4', 'Sol Ring', 'normal'),
          ('00000000-0000-0000-0000-00000000aaaa', 'Sol Ring', 'art_series'),
          ('60658907-dd63-478d-a66a-123e4e9d2a00', 'Bloodline Keeper // Lord of Lineage', 'transform'),
          ('a85e6c5e-3895-4b26-a6e6-e08c2bc00821', 'Bloodline Keeper // Bloodline Keeper', 'art_series'),
          ('18303862-4726-4136-814f-157aa7006579', 'Fire // Ice', 'split'),
          ('11111111-1111-1111-1111-111111111111', 'Goblin', 'token'),
          ('22222222-2222-2222-2222-222222222222', 'Elspeth Emblem', 'emblem'),
          ('33333333-3333-3333-3333-333333333333', 'Treasure // Food', 'double_faced_token'),
          ('44444444-4444-4444-4444-444444444444', 'Front Card', 'front_card'),
          ('bbbbbbbb-0000-0000-0000-000000000002', 'Everythingamajig', 'normal'),
          ('bbbbbbbb-0000-0000-0000-000000000001', 'Everythingamajig', 'normal');
        INSERT INTO card_faces VALUES
          ('60658907-dd63-478d-a66a-123e4e9d2a00', 0, 'Bloodline Keeper'),
          ('60658907-dd63-478d-a66a-123e4e9d2a00', 1, 'Lord of Lineage'),
          ('a85e6c5e-3895-4b26-a6e6-e08c2bc00821', 0, 'Bloodline Keeper'),
          ('a85e6c5e-3895-4b26-a6e6-e08c2bc00821', 1, 'Bloodline Keeper'),
          ('18303862-4726-4136-814f-157aa7006579', 0, 'Fire'),
          ('18303862-4726-4136-814f-157aa7006579', 1, 'Ice'),
          ('33333333-3333-3333-3333-333333333333', 0, 'Treasure');";

    #[test]
    fn test_image_sqlite_ids_resolve_exact_names_and_front_faces_and_skip_non_game_layouts() {
        let temp = TempStore::new("sqlite");
        let ids = SqliteIds::open(&sqlite_fixture(&temp, FIXTURE_SCHEMA)).expect("opens");
        let id = |name: &str| ids.scryfall_id(name);
        // Exact name; the art-series row of the same name is never the answer.
        assert_eq!(id("Sol Ring").as_deref(), Some(SOL_RING_ID));
        // The full two-part name, and the front face the engine names the card by.
        let keeper = Some("60658907-dd63-478d-a66a-123e4e9d2a00".to_string());
        assert_eq!(id("Bloodline Keeper // Lord of Lineage"), keeper);
        assert_eq!(id("Bloodline Keeper"), keeper);
        assert_eq!(
            id("Fire").as_deref(),
            Some("18303862-4726-4136-814f-157aa7006579")
        );
        // A back face is not a card name, and non-game objects are not cards.
        for absent in [
            "Lord of Lineage",
            "Ice",
            "Goblin",
            "Elspeth Emblem",
            "Treasure // Food",
            "Treasure",
            "Front Card",
            "Bloodline Keeper // Bloodline Keeper",
            "sol ring",
            "Sol Ring ",
            "Steel Guardian",
            "",
            "' OR 1=1 --",
        ] {
            assert_eq!(id(absent), None, "{absent:?}");
        }
        // Same-named rows: the pick is the lowest id, every time.
        assert_eq!(
            id("Everythingamajig").as_deref(),
            Some("bbbbbbbb-0000-0000-0000-000000000001")
        );
    }

    #[test]
    fn test_image_sqlite_ids_open_read_only_and_refuse_a_missing_or_drifted_file() {
        let temp = TempStore::new("sqlite-bad");
        let missing = temp.root.join("nope.sqlite");
        assert!(SqliteIds::open(&missing).is_err());
        assert!(!missing.exists(), "a read-only open creates nothing");
        // A renamed column (the heads-up rule's case) fails the open, not a query.
        let drifted = sqlite_fixture(
            &temp,
            "CREATE TABLE cards (scryfall_id TEXT, name TEXT, layout TEXT);
             CREATE TABLE card_faces (card_id TEXT, face_index INTEGER, name TEXT);",
        );
        assert!(SqliteIds::open(&drifted).is_err());
        // Not a database at all.
        let junk = temp.root.join("junk.sqlite");
        std::fs::write(&junk, b"this is not sqlite").unwrap();
        assert!(SqliteIds::open(&junk).is_err());
        // The server's opener turns each of those into "no ids", reported as such.
        let (ids, label) = open_card_ids(&missing);
        assert_eq!(label, "none");
        assert_eq!(ids.scryfall_id("Sol Ring"), None);
        // A good file is written to by nobody: the connection refuses.
        let temp2 = TempStore::new("sqlite-ro");
        let good = sqlite_fixture(&temp2, FIXTURE_SCHEMA);
        let ids = SqliteIds::open(&good).unwrap();
        assert!(ids.conn.execute("DELETE FROM cards", []).is_err());
        let (boxed, label) = open_card_ids(&good);
        assert_eq!(label, good.display().to_string());
        assert_eq!(boxed.scryfall_id("Sol Ring").as_deref(), Some(SOL_RING_ID));
    }

    // ── the worker, the retry queue and the retry route ──

    /// A [`FakeNet`] the test keeps a handle on while the worker thread uses it.
    #[derive(Clone, Default)]
    struct SharedNet(Arc<Mutex<FakeNet>>);

    impl Net for SharedNet {
        fn get(&mut self, url: &str, accept: &str) -> Result<Vec<u8>, FetchError> {
            self.0.lock().unwrap().get(url, accept)
        }

        fn pause(&mut self, duration: Duration) {
            self.0.lock().unwrap().pause(duration);
        }
    }

    /// A fetching service over a scripted network and the Sol Ring id map.
    fn fake_service(temp: &TempStore, net: &SharedNet) -> ImageService {
        let net = net.clone();
        ImageService::with_worker(
            temp.root.clone(),
            Box::new(sol_ring_ids()),
            "fake".to_string(),
            move || net,
        )
    }

    /// Wait (bounded) for the worker to drain its queue.
    fn drained(service: &ImageService) -> PrefetchStatus {
        for _ in 0..2000 {
            let status = service.status();
            if status.queued == 0 {
                return status;
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        panic!("the worker never drained: {:?}", service.status());
    }

    fn fail_small(net: &SharedNet, code: u16) {
        let mut net = net.0.lock().unwrap();
        for _ in 0..MAX_ATTEMPTS {
            net.script(
                &id_url(SOL_RING_ID, Version::Small),
                Err(FetchError::Status(code)),
            );
        }
    }

    fn serve_all_sizes(net: &SharedNet) {
        let mut net = net.0.lock().unwrap();
        for version in Version::ALL {
            net.script(&id_url(SOL_RING_ID, version), Ok(JPEG.to_vec()));
        }
    }

    #[test]
    fn test_image_worker_retries_a_failed_card_on_the_next_deal_that_names_it() {
        let temp = TempStore::new("worker-redeal");
        let net = SharedNet::default();
        fail_small(&net, 429);
        let service = fake_service(&temp, &net);
        // Deal one: Sol Ring exhausts its attempts on a 429; the other name is
        // unknown to the id map and to (the fake) Scryfall.
        let deal = vec!["Sol Ring".to_string(), "Steel Guardian".to_string()];
        assert_eq!(service.enqueue(deal.clone()), 2);
        let status = drained(&service);
        assert_eq!(status.failed, vec!["Sol Ring", "Steel Guardian"]);
        assert_eq!(
            status.retry_queue,
            vec!["Sol Ring"],
            "an unknown name is not worth retrying"
        );
        assert_eq!((status.by_id, status.named_lookups), (1, 1));
        assert_eq!(status.card_ids, "fake");
        // Deal two names it again; the CDN has recovered.
        serve_all_sizes(&net);
        let api_before = net.0.lock().unwrap().calls_to("https://api.scryfall.com/");
        assert_eq!(service.enqueue(deal), 2);
        let status = drained(&service);
        assert_eq!(status.failed, vec!["Steel Guardian"]);
        assert!(status.retry_queue.is_empty());
        assert_eq!(status.fetched, 1);
        assert!(missing_versions(service.store(), "Sol Ring").is_empty());
        assert_eq!(
            net.0.lock().unwrap().calls_to("https://api.scryfall.com/"),
            api_before,
            "the refused name is not asked about again, and the retry is CDN-only"
        );
    }

    #[test]
    fn test_image_retry_failed_requeues_the_retry_queue_and_only_it() {
        let temp = TempStore::new("worker-retry");
        let net = SharedNet::default();
        fail_small(&net, 503);
        let service = fake_service(&temp, &net);
        service.enqueue(vec!["Sol Ring".to_string(), "Steel Guardian".to_string()]);
        let status = drained(&service);
        assert_eq!(status.retry_queue, vec!["Sol Ring"]);
        // Still down: the retry fails again and the card stays queued for retry.
        fail_small(&net, 503);
        assert_eq!(service.retry_failed(), 1, "one card, not the unknown name");
        assert_eq!(drained(&service).retry_queue, vec!["Sol Ring"]);
        // Recovered.
        serve_all_sizes(&net);
        assert_eq!(service.retry_failed(), 1);
        let status = drained(&service);
        assert!(status.retry_queue.is_empty());
        assert_eq!(status.failed, vec!["Steel Guardian"]);
        assert!(missing_versions(service.store(), "Sol Ring").is_empty());
        assert_eq!(service.retry_failed(), 0, "nothing left to retry");
        assert_eq!(
            net.0.lock().unwrap().calls_to("https://api.scryfall.com/"),
            1,
            "only the unknown name's one lookup ever reached the API"
        );
    }

    async fn post(state: &AppState, uri: &str) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let response = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("router is infallible");
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, headers, body.to_vec())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_retry_route_requeues_failed_cards_and_reports_counts_only() {
        let temp = TempStore::new("route-retry");
        let net = SharedNet::default();
        fail_small(&net, 429);
        let service = fake_service(&temp, &net);
        service.enqueue(vec!["Sol Ring".to_string()]);
        drained(&service);
        let state = AppState::new(defaults()).with_images(service.clone());

        let (status, _, body) = get(&state, "/api/card-image/status").await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["failed"], 1);
        assert_eq!(json["retry_queue"], 1, "the status route reports the queue");
        assert_eq!(json["by_id"], 1);
        assert_eq!(json["named_lookups"], 0);
        assert_eq!(json["card_ids"], "fake");

        serve_all_sizes(&net);
        let (status, headers, body) = post(&state, "/api/card-image/retry").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        let text = String::from_utf8(body).unwrap();
        assert!(!text.contains("Sol Ring"), "counts only: {text}");
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["requeued"], 1);
        let status = drained(&service);
        assert!(status.failed.is_empty() && status.retry_queue.is_empty());
        assert!(missing_versions(service.store(), "Sol Ring").is_empty());

        // Nothing to retry, and a GET is not the retry route.
        let (_, _, body) = post(&state, "/api/card-image/retry").await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["requeued"], 0);
        let (status, _, _) = get(&state, "/api/card-image/retry").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_retry_route_is_a_no_op_on_a_serve_only_server() {
        let state = AppState::new(defaults());
        let (status, _, body) = post(&state, "/api/card-image/retry").await;
        assert_eq!(status, StatusCode::OK);
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["requeued"], 0);
        assert_eq!(json["enabled"], false);
        assert_eq!(json["retry_queue"], 0);
        assert_eq!(json["card_ids"], "none");
        assert!(!state.images.store().root().exists());
    }

    #[test]
    fn test_image_ensure_card_never_stores_a_non_jpeg_body() {
        let temp = TempStore::new("nonjpeg");
        let store = temp.store();
        let mut net = FakeNet::default();
        net.script(
            &named_card_url("Sol Ring"),
            Ok(SOL_RING_JSON.as_bytes().to_vec()),
        );
        net.script(
            "https://cards.scryfall.io/small/front/a/b/sol.jpg?1",
            Ok(b"<html>rate limited</html>".to_vec()),
        );
        let outcome = ensure_card(&mut net, &store, "Sol Ring");
        assert_eq!(
            outcome,
            CardOutcome::Failed("small image is not a JPEG".to_string())
        );
        assert!(!store.has("Sol Ring", Version::Small));
    }

    #[test]
    fn test_image_prefetch_all_summarises_fetched_present_and_failed() {
        let temp = TempStore::new("summary");
        let store = temp.store();
        for version in Version::ALL {
            store.write_atomic("Command Tower", version, JPEG).unwrap();
        }
        let mut net = sol_ring_net();
        let names = vec![
            "Command Tower".to_string(),
            "No Such Card Zzz".to_string(),
            "Sol Ring".to_string(),
        ];
        let mut lines = Vec::new();
        let summary = prefetch_all(&mut net, &store, &NoIds, &names, &mut |l| {
            lines.push(l.to_string())
        });
        assert_eq!(summary.fetched, vec!["Sol Ring".to_string()]);
        assert_eq!(summary.present, vec!["Command Tower".to_string()]);
        assert_eq!(summary.failed.len(), 1);
        assert_eq!(summary.failed[0].0, "No Such Card Zzz");
        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("[2/3] FAILED"), "{lines:?}");
    }

    // ── decklists ──

    #[test]
    fn test_image_decklist_parser_handles_counts_comments_and_blank_lines() {
        let text = "# my deck\n\
                    // commander\n\
                    1 Atraxa, Praetors' Voice\n\
                    \n\
                    1x Sol Ring\n\
                    12X Forest\n\
                    \t 3   Fire // Ice  \r\n\
                    Command Tower\n\
                    1 Sol Ring\n\
                    2 1996 World Champion\n\
                    7th Fleet\n\
                    x Marks the Spot\n";
        assert_eq!(
            parse_decklist(text),
            vec![
                "1996 World Champion",
                "7th Fleet",
                "Atraxa, Praetors' Voice",
                "Command Tower",
                "Fire // Ice",
                "Forest",
                "Sol Ring",
                "x Marks the Spot",
            ]
        );
        assert!(parse_decklist("").is_empty());
        assert!(parse_decklist("#only\n//comments\n\n").is_empty());
    }

    #[test]
    fn test_image_name_list_splits_on_semicolons_when_present_else_commas() {
        assert_eq!(
            parse_name_list("Sol Ring, Forest ,,Sol Ring"),
            vec!["Forest", "Sol Ring"]
        );
        assert_eq!(
            parse_name_list("Atraxa, Praetors' Voice; Sol Ring"),
            vec!["Atraxa, Praetors' Voice", "Sol Ring"]
        );
        assert!(parse_name_list(" , ").is_empty());
    }

    #[test]
    fn test_image_prefetch_argument_reads_a_file_when_one_exists() {
        let temp = TempStore::new("arg");
        std::fs::create_dir_all(&temp.root).unwrap();
        let file = temp.root.join("deck.txt");
        std::fs::write(&file, "1 Sol Ring\n# c\n1 Forest\n").unwrap();
        assert_eq!(
            names_from_arg(file.to_str().unwrap()).unwrap(),
            vec!["Forest", "Sol Ring"]
        );
        assert_eq!(
            names_from_arg("Sol Ring,Forest").unwrap(),
            vec!["Forest", "Sol Ring"]
        );
        // A path that exists but is not readable text is the one hard error.
        assert!(names_from_arg(temp.root.to_str().unwrap()).is_err());
    }

    // ── the service and the deck hook ──

    fn defaults() -> NewGameDefaults {
        NewGameDefaults {
            players: 4,
            bot: BotKind::Heuristic,
            seed: 0,
        }
    }

    #[test]
    fn test_image_default_app_state_serves_only_and_cannot_queue_a_fetch() {
        let state = AppState::new(defaults());
        assert!(!state.images.prefetch_enabled());
        assert_eq!(state.images.enqueue(vec!["Sol Ring".to_string()]), 0);
        let status = state.images.status();
        assert!(!status.enabled);
        assert_eq!((status.queued, status.fetched, status.present), (0, 0, 0));
        assert!(status.failed.is_empty());
        assert!(
            !state.images.store().root().exists(),
            "the default test store must be a path that does not exist"
        );
    }

    #[test]
    fn test_image_status_bookkeeping_counts_cards_and_clears_a_recovered_failure() {
        let mut status = PrefetchStatus {
            queued: 4,
            ..PrefetchStatus::default()
        };
        record(
            &mut status,
            "B",
            &CardOutcome::Failed("x".to_string()),
            Route::NamedLookup,
        );
        record(
            &mut status,
            "B",
            &CardOutcome::Failed("x".to_string()),
            Route::NamedLookup,
        );
        record(&mut status, "A", &CardOutcome::Present, Route::Store);
        assert_eq!((status.queued, status.present, status.fetched), (1, 1, 0));
        assert_eq!(status.failed, vec!["B"]);
        record(&mut status, "B", &CardOutcome::Fetched(3), Route::CdnById);
        record(&mut status, "C", &CardOutcome::Fetched(1), Route::CdnById);
        assert_eq!((status.queued, status.fetched), (0, 2));
        assert!(status.failed.is_empty());
    }

    #[test]
    fn test_image_deck_card_names_cover_every_seat_of_a_resolved_table() {
        let cfg = session::config_for(defaults()).unwrap();
        assert!(
            deck_card_names(&cfg).is_empty(),
            "an unresolved recipe names no cards"
        );
        let play = session::new_game(cfg, 0).unwrap();
        let names = deck_card_names(&play.cfg);
        let DeckSource::Fixed(decks) = &play.cfg.decks else {
            panic!("new_game pins the decks");
        };
        assert_eq!(decks.len(), 4);
        let by_id: HashMap<_, _> = mtg_engine::all_cards()
            .into_iter()
            .map(|d| (d.card_id, d.name))
            .collect();
        let mut expected = BTreeSet::new();
        for (_, deck) in decks {
            expected.insert(by_id[&deck.commander].clone());
            for id in &deck.main_deck {
                expected.insert(by_id[id].clone());
            }
        }
        assert!(expected.len() > 4, "four decks name more than four cards");
        assert_eq!(names, expected.into_iter().collect::<Vec<_>>());
    }

    // ── routes ──

    fn app(state: AppState) -> axum::Router {
        crate::build_router(state, &PathBuf::from("nonexistent_dist"))
    }

    async fn get(state: &AppState, uri: &str) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
        let response = app(state.clone())
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .expect("router is infallible");
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        (status, headers, body.to_vec())
    }

    fn state_over(temp: &TempStore) -> AppState {
        AppState::new(defaults()).with_images(ImageService::serve_only(&temp.root))
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_route_serves_a_stored_jpeg_with_a_long_cache() {
        let temp = TempStore::new("route-hit");
        let name = "Atraxa, Praetors' Voice";
        temp.store()
            .write_atomic(name, Version::Normal, JPEG)
            .unwrap();
        let state = state_over(&temp);
        let uri = format!(
            "/api/card-image?name={}&version=normal",
            percent_encode(name)
        );
        let (status, headers, body) = get(&state, &uri).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_TYPE], "image/jpeg");
        assert_eq!(headers[header::CACHE_CONTROL], "public, max-age=2592000");
        assert_eq!(body, JPEG);
        // `+` for a space, as a browser's URLSearchParams writes it.
        let (status, _, _) = get(
            &state,
            "/api/card-image?version=normal&name=Atraxa%2C+Praetors%27+Voice",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_route_answers_404_no_store_for_a_miss() {
        let temp = TempStore::new("route-miss");
        temp.store()
            .write_atomic("Sol Ring", Version::Small, JPEG)
            .unwrap();
        let state = state_over(&temp);
        for uri in [
            "/api/card-image?name=Sol%20Ring&version=large",
            "/api/card-image?name=Forest&version=small",
            "/api/card-image?name=..%2F..%2Fetc%2Fpasswd&version=small",
            "/api/card-image?name=..%2Fsmall%2Fsol-ring&version=normal",
        ] {
            let (status, headers, body) = get(&state, uri).await;
            assert_eq!(status, StatusCode::NOT_FOUND, "{uri}");
            assert_eq!(headers[header::CACHE_CONTROL], "no-store", "{uri}");
            let json: serde_json::Value = serde_json::from_slice(&body).expect("envelope");
            assert_eq!(json["kind"], "image_not_in_store", "{uri}");
            assert!(json["error"].is_string());
        }
        // Once the file exists the very next request is a hit.
        temp.store()
            .write_atomic("Forest", Version::Small, JPEG)
            .unwrap();
        let (status, _, _) = get(&state, "/api/card-image?name=Forest&version=small").await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_route_answers_400_for_a_bad_version_or_missing_name() {
        let temp = TempStore::new("route-400");
        temp.store()
            .write_atomic("Sol Ring", Version::Small, JPEG)
            .unwrap();
        let state = state_over(&temp);
        for uri in [
            "/api/card-image",
            "/api/card-image?version=small",
            "/api/card-image?name=&version=small",
            "/api/card-image?name=%20&version=small",
            "/api/card-image?name=Sol%20Ring",
            "/api/card-image?name=Sol%20Ring&version=png",
            "/api/card-image?name=Sol%20Ring&version=Small",
            "/api/card-image?name=Sol%20Ring&version=..%2Fsmall",
        ] {
            let (status, headers, body) = get(&state, uri).await;
            assert_eq!(status, StatusCode::BAD_REQUEST, "{uri}");
            assert_eq!(headers[header::CONTENT_TYPE], "application/json", "{uri}");
            let json: serde_json::Value = serde_json::from_slice(&body).expect("envelope");
            assert_eq!(json["kind"], "bad_params", "{uri}");
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_card_image_route_works_with_no_session_and_no_store_directory() {
        // No game was ever started and the store path does not exist: the image
        // routes read no session, so neither matters.
        let state = AppState::new(defaults());
        let (status, _, _) = get(&state, "/api/card-image?name=Sol%20Ring&version=small").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let (status, _, _) = get(&state, "/api/game").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "there is no session");
        let (status, headers, body) = get(&state, "/api/card-image/status").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["enabled"], false);
        assert_eq!(json["queued"], 0);
        assert_eq!(json["fetched"], 0);
        assert_eq!(json["present"], 0);
        assert_eq!(json["failed"], 0, "`failed` is a count, never a name list");
        assert!(json["store"].is_string());
    }

    #[test]
    fn test_image_cdn_downloads_refuse_redirects_and_the_api_call_does_not() {
        // The host check in `image_urls` covers the first URL only; the CDN
        // agent must not follow a 3xx anywhere else. Built, never used: no
        // request is made here.
        let cdn = cdn_config();
        assert_eq!(cdn.max_redirects(), 0, "image downloads follow no redirect");
        assert!(cdn.https_only());
        assert!(refuses_redirects(
            "https://cards.scryfall.io/small/front/a/b/abc.jpg"
        ));
        assert!(!refuses_redirects(&named_card_url("Sol Ring")));
        // Every URL `image_urls` lets through is one the redirect-refusing agent takes.
        let card = serde_json::json!({"image_uris": {
            "small": "https://cards.scryfall.io/small/x.jpg",
            "normal": "https://cards.scryfall.io/normal/x.jpg",
            "large": "https://cards.scryfall.io/large/x.jpg",
        }});
        let urls = image_urls(&card).expect("three CDN urls");
        assert!(urls.iter().all(|(_, url)| refuses_redirects(url)));
    }

    #[test]
    fn test_image_status_body_carries_the_failed_count_and_no_card_name() {
        // The failed names are cards of the table's decks, bot seats included:
        // the body may say how many, never which (Architecture Invariant 7).
        let mut status = PrefetchStatus {
            queued: 2,
            ..PrefetchStatus::default()
        };
        record(
            &mut status,
            "Zzyzx Hidden Bot Card",
            &CardOutcome::Failed("HTTP 429".to_string()),
            Route::CdnById,
        );
        record(
            &mut status,
            "Another Secret Name",
            &CardOutcome::Failed("HTTP 429".to_string()),
            Route::CdnById,
        );
        assert_eq!(status.failed.len(), 2, "the names stay in memory");
        assert_eq!(status.retry_queue.len(), 2);
        let json = serde_json::to_value(&status).unwrap();
        assert_eq!(json["failed"], 2);
        assert_eq!(json["retry_queue"], 2, "a count, like `failed`");
        let text = json.to_string();
        assert!(
            !text.contains("Zzyzx") && !text.contains("Secret"),
            "{text}"
        );
        assert!(json.as_object().unwrap().values().all(|v| !v.is_array()));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_starting_a_game_on_a_default_state_queues_no_image_fetch() {
        let temp = TempStore::new("route-game");
        let state = state_over(&temp);
        let response = app(state.clone())
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/game")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("router is infallible");
        assert_eq!(response.status(), StatusCode::OK);
        let status = state.images.status();
        assert!(!status.enabled);
        assert_eq!((status.queued, status.fetched, status.present), (0, 0, 0));
        assert!(
            !temp.root.exists(),
            "starting a game wrote nothing to the store"
        );
    }
}
