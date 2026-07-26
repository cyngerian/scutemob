# UI Feedback Ledger — play-server frontend (M11-local+)

> Standing channel for the user's hands-on feedback about the play UI (and, where
> relevant, the replay viewer, since they share components). Created 2026-07-26,
> ahead of the first playable build (M11 session 6).
>
> **Protocol**
> - The user comments in any coordinator session ("the hand is too cramped",
>   "I couldn't tell whose turn it was") — or edits this file directly. Either way
>   the coordinator records it here as a dated entry with status `new`.
> - The coordinator triages `new` entries at each session start and on request:
>   duplicates merged, each kept item classified **bug** (wrong behavior) /
>   **usability** (correct but confusing) / **polish** (M13-deferred visual), then
>   either filed as an ESM task (status `filed → scutemob-N`), batched for the next
>   UI session dispatch (`queued`), or explicitly deferred to M13/M14 (`deferred`)
>   with a one-line reason.
> - Bugs in *rules behavior* discovered while playing do NOT belong here — those go
>   through the engine's normal issue flow (they are engine findings, not UI notes).
>   The coordinator reroutes them.
> - Keep entries short; this is an inbox, not a design doc. Design discussion that
>   an entry provokes lives in the task it gets filed to.

## Format

```
### YYYY-MM-DD — <short title>   [status: new|queued|filed → task|deferred|done]
What happened / what was expected. One entry per distinct observation.
```

---

## Entries

(none yet — first playable build not landed)
