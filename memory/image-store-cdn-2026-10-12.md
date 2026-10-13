# Image store: prefetch from the Scryfall CDN by card id; retry queue (`scutemob-270`, 2026-10-12)

Change class: row 3 (tooling; `tools/play-server` only). No source gate added, so row 4 does not
apply. No engine, card-types, view-model or wire change (PROTOCOL 44 / HASH 85 untouched).
Files: `tools/play-server/src/images.rs`, `src/main.rs`, `Cargo.toml` (+`rusqlite`, `card-db`'s line
verbatim; `Cargo.lock` gains one edge line), README, `docs/play-v3-playtest-guide.md` §1.

## Why

C3's prefetch made one `api.scryfall.com/cards/named` call per card before downloading. That call is
the rate-limited one: the owner's first deal on 2026-10-12 got **41 cards in ~4 minutes** under 429
backoff. The CDN answered by id at ~0.15 s per image throughout.

## URL derivation

`cdn_image_url(id, version)` = `https://cards.scryfall.io/<size>/front/<id[0]>/<id[1]>/<id>.jpg`.
Pure. The id is shape-checked (lowercase hyphenated UUID, 8-4-4-4-12) before it is spliced into a
URL — it comes out of a file on disk; a malformed one takes the fallback and never reaches a URL.
Every id-derived URL starts with the CDN prefix, so it is fetched by the redirect-refusing agent.

## The sqlite read (`SqliteIds`)

- Opened `SQLITE_OPEN_READ_ONLY` (a missing file is an error, never created). Two prepared queries,
  both compiled at open as the schema probe. No dependency on `card-db`'s schema module; the column
  heads-up rule (decisions 2026-09-14) is named in a comment above the SQL.
- Query 1: `cards.id` by exact `cards.name`, excluding layouts `art_series, token,
  double_faced_token, emblem, front_card`; `ORDER BY id LIMIT 1` (17 Un-set names have several rows).
- Query 2, only when query 1 finds nothing: the card whose `card_faces` row with `face_index = 0`
  has that name (same layout exclusion).
- Path: `--cards-sqlite`, else `$SCUTEMOB_CARDS_SQLITE`, else `cards.sqlite` in the cwd, else at the
  workspace root the binary was built from. Unusable file (missing, not a database, renamed column)
  → `NoIds`, ONE loud stderr line, status `card_ids: "none"`, every card takes the named lookup.
  NOTE for worktrees: `cards.sqlite` is gitignored, so a server run from a worktree needs the flag.

## DFC / split / adventure handling

Brief said "`card_faces` rows if the front path 404s". What was found and done instead:
- The `front/` path by the CARD's id **does not 404** for a double-faced card (faces share the card
  id; `front/` is the front face). Checked live: Bloodline Keeper (`60658907-…`, transform) small →
  200, 12,106 B; Fire // Ice (`18303862-…`, split) normal → 200, 82,432 B.
- What DFCs actually need is the NAME match: the engine names such a card by its front face
  (`Bloodline Keeper`) while `cards.name` is `Front // Back`. That is query 2.
- Whole def pool against this machine's `cards.sqlite` (throwaway test, not committed): of 1,803
  names, **1,788** resolve by `cards.name`, **13** by front face, **2** not at all (`Poisonous
  Viper`, `Steel Guardian` — synthetic test cards; they take the fallback and Scryfall refuses them).
- An id path that does 404 (a stale id) falls back to the named lookup, once per card; sizes the id
  path already stored are not downloaded again.

## Fetch procedure (`ensure_card_via`)

Id known → per missing size: `Net::space(100 ms)` then the CDN GET with the existing bounded retry
(`backoff_delay` unchanged). Zero `api.scryfall.com` requests. 429/5xx exhausted → `Failed`
(retryable); it does NOT then go to the API (a rate-limited CDN is no reason to hit the rate-limited
API). Id unknown, or an id path 404 → the old named-lookup route, unchanged.

Spacing: `CDN_SPACING` = 100 ms between the STARTS of id-derived downloads (≤10 requests/s).
`HttpNet::space` sleeps only what is left of the gap since the previous one; the trait default
(what the test fake records) is a full pause.

## Retry design

- `PrefetchStatus.retry_queue`: failed names whose reason is anything but "Scryfall knows no card by
  this exact name". Serialized as a COUNT, like `failed` (names are deck cards, bot seats included).
- **Next deal**: `post_game` enqueues every deck name as before; the worker runs a retry-queue name
  through the fetch again (only refused-unknown names are skipped). Now pinned by a test on the
  worker over a scripted `Net`.
- **`POST /api/card-image/retry`** → `ImageService::retry_failed()` re-queues the whole retry queue;
  answers the status body plus `requeued`. No session lock, no fetch on the request thread, `0` on a
  serve-only server. A name leaves the queue when it succeeds (`Present` now also clears a failure).
- Status also gains `by_id`, `named_lookups`, `card_ids`.

## Measured (criterion 3) — cold store, this machine, 2026-10-12

Probe server on 3041, scratch store under the session scratchpad, `--seed 0 --players 4`,
`--cards-sqlite` at the main checkout's file. Port 3040 and `~/.cache/scutemob/card-images` untouched.

| Run | Cards | Images | To `queued: 0` | `by_id` | `named_lookups` | `failed` |
|---|---|---|---|---|---|---|
| C3, 2026-10-12 (owner's first deal, api lookups) | 41 fetched | — | ~4 min, 429 backoff | — | every card | — |
| C3 notes figure (seeds 0 + 3) | 384 | 1,152 | 1,254 s | — | every card | 6 (429) |
| **this task, run 1** | 238 | 714 | **72.6 s** (first→last file write) | 238 | 0 | 0 |
| **this task, run 2** (store deleted, re-dealt) | 238 | 714 | **71.7 s** (wall clock from the POST) | 238 | 0 | 0 |

714 files, all JPEG, no `.part` leftovers, 61 MB; server stderr carried no `card images:` line. The
time is the spacing: 714 × 100 ms. Run 1's wall clock was lost (the timing line used `bc`, which is
not installed), hence the file-time span; run 2 is a true wall clock. Scratch store and log deleted;
the probe server was stopped by its own pid (found by port 3041).

The guide said "≈400 cards for four seats"; seed 0 is 238 distinct names. Guide corrected.

## Tests (all in `images.rs`; none opens a socket)

Added 15: URL derivation; malformed ids refused; by-id = 3 CDN calls / 0 API calls / spacing; by-id
partial store; id 404 → named lookup exactly once; 404 mid-card; name absent → named lookup; by-id
429 exhausts without an API call; malformed id → fallback; sqlite resolve (names, front faces,
excluded layouts, duplicates); sqlite missing / drifted / junk / read-only; worker retries on the
next deal; `retry_failed`; retry route; retry route serve-only. The pre-existing named-route tests
are unmodified apart from `record`/`prefetch_all` gaining an argument; `ensure_card` is now a test
helper (`ensure_card_via` with `NoIds`).

## Observations / LOW

- The named-lookup route still downloads its three sizes back to back with no 100 ms spacing (as in
  C3; pinned by `…makes_one_api_call_then_three_cdn_downloads`). It is now reached only for names
  the sqlite lacks, so it was left alone.
- `refused` (unknown names) is per-process; nothing clears it but a restart. Unchanged from C3.
