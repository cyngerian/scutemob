# scutemob-270 — image store CDN-by-id + retry: task list

Change class: row 3 (tooling; play-server only). No source gate planned, so row 4 does not apply.
Updated live; the same list is reposted as an `esm task comment` at each milestone.

- [x] 1. Read worker.md, brief, images.rs, main.rs wiring, conventions change-class table, sqlite schema
- [x] 2. URL derivation: `cdn_image_url(id, version)` (pure; id shape-checked) + test
- [x] 3. sqlite read: `CardIds` trait, `SqliteIds` (read-only, two-column query by exact name, layout exclusion, front-face `card_faces` match), missing/unusable file = no ids, logged once
- [x] 4. `ensure_card` CDN-by-id path: zero api.scryfall.com calls when the id is known; ≥100 ms CDN spacing
- [x] 5. Fallback: name absent from sqlite → named lookup; id path 404 → named lookup exactly once
- [x] 6. Retry: failed (non-unknown) names kept as the retry queue; re-run on the next deal that needs them; `ImageService::retry_failed`
- [x] 7. Route `POST /api/card-image/retry` wired in main.rs
- [x] 8. Status fields: `retry_queue`, `by_id`, `named_lookups`, `card_ids` (counts only; no names)
- [x] 9. Flag `--cards-sqlite` / env `SCUTEMOB_CARDS_SQLITE` + default; CLI `--prefetch-images` uses the ids too
- [x] 10. Test: URL derivation + malformed id refused
- [x] 11. Test: SqliteIds (exact name, excluded layouts, front face, absent, missing file, drifted schema)
- [x] 12. Test: FakeNet — id known → 3 CDN calls, 0 API calls, spacing pauses
- [x] 13. Test: FakeNet — id path 404 → named lookup exactly once, card still stored
- [x] 14. Test: FakeNet — name absent from ids → named lookup (counted)
- [x] 15. Test: worker — failed card retried on the next enqueue that names it; retry route re-queues; status reports the queue; unknown names not re-queued
- [x] 16. Test: status body still carries counts only (no names) with the new fields
- [x] 17. Docs: README route + flag lines; playtest guide §1 "deal again" replaced
- [x] 18. Cold-store measurement: scratch store, probe server on 3041, four-seat deal, time to `queued == 0`; scratch deleted
- [ ] 19. Gates: `cargo test --workspace --no-fail-fast` to a file (delta by NAME vs 5,407), clippy, fmt --check, check-defs-fmt.sh
- [x] 20. Notes `memory/image-store-cdn-2026-10-12.md` + CHANGELOG entry
- [ ] 21. /review; HIGH/MEDIUM fixed in-cycle, LOW logged
- [ ] 22. Satisfy criteria as met (8623, 8624, 8625, 8626); signal-ready; session end
