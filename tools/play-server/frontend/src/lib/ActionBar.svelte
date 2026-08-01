<script>
  /**
   * ActionBar — the pending decision, rendered as buttons.
   *
   * M11-local Session 6 (`memory/m11-session-plan.md` §4, item 5).
   *
   * Props:
   *   decision (DecisionView|null) — `{ seq, kind, player, actions }`
   *   loading (bool)              — a request is in flight
   *   error ({message,kind,status}|null)
   *   emptyReason (string)        — why there is no decision, supplied by the caller
   *   onAct (fn(index, params))   — submit an action
   *   onRefresh (fn)              — re-read the seat view
   *   onDismissError (fn)
   *   onCancel (fn)               — Escape: clear the caller's pending picker
   *
   * # `index`, and nothing else
   *
   * A submission is `{seq, action_index, params}`. The server maps the index back
   * through the `PendingDecision` it is still holding, so no engine type is a wire
   * type (`view.rs` module doc, "`LegalAction` is NEVER serialized"). `params` is
   * `{}` this session; Session 7 adds the target / attacker / blocker / X / mode
   * pickers that fill it.
   */
  const {
    decision = null,
    loading = false,
    error = null,
    emptyReason = '',
    onAct = null,
    onRefresh = null,
    onDismissError = null,
    onCancel = null,
  } = $props();

  const actions = $derived(decision?.actions ?? []);

  /**
   * `PassPriority` and `Concede` are pulled into their own group on the right so
   * "pass" is always in the same place, however long the action list gets — plan
   * item 5 asks for it to be easy to find. The original `index` travels with each
   * option, so the split never affects what is submitted.
   */
  const controlKinds = ['PassPriority', 'Concede'];
  const plays = $derived(actions.filter((a) => !controlKinds.includes(a.kind)));
  const controls = $derived(actions.filter((a) => controlKinds.includes(a.kind)));

  /** Found by `kind`, never by a hardcoded index — the list order is the server's. */
  const passAction = $derived(actions.find((a) => a.kind === 'PassPriority') ?? null);

  /**
   * Combat declarations submit an **empty** set until Session 7's pickers land,
   * and the button says so rather than letting it happen quietly.
   *
   * `params.rs` maps `LegalAction::DeclareAttackers` with default params straight
   * to `Command::DeclareAttackers { attackers: vec![] }` (and likewise for
   * blockers), which is a legal and irreversible "I attack with nothing" for that
   * combat. That is a different animal from the targeted-spell case, which the
   * engine *refuses* with a loud 422 under CR 601.2c — here nothing complains and
   * the human's combat step is simply gone.
   *
   * The buttons stay **enabled** deliberately. At a `DeclareAttackers` decision
   * the declaration is typically the only option offered, so disabling it would
   * deadlock the game rather than protect anyone; CR 508.1 also makes declaring
   * no attackers a legal choice. The fix available to S6 is honesty about what the
   * click does, not prevention.
   */
  const EMPTY_SET_KINDS = ['DeclareAttackers', 'DeclareBlockers'];
  function declaresEmptySet(kind) {
    return EMPTY_SET_KINDS.includes(kind);
  }

  function submit(option) {
    if (loading) return;
    onAct?.(option.index, {});
  }

  /**
   * Keyboard shortcuts (plan item 5): space = pass priority, Escape = cancel the
   * pending picker and dismiss the error strip.
   *
   * Bound on `window` in an `$effect` with cleanup. The handler reads `decision`
   * and `loading` when it fires rather than when the effect runs, so the effect
   * registers once instead of re-binding on every state change while still seeing
   * current values.
   */
  $effect(() => {
    function isTyping(target) {
      if (!target) return false;
      if (target.isContentEditable) return true;
      return ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName);
    }

    function onKeyDown(event) {
      // Typing a seed into a field must not pass priority.
      if (isTyping(event.target)) return;
      if (event.ctrlKey || event.metaKey || event.altKey) return;

      if (event.key === ' ' || event.code === 'Space') {
        // Always prevent the default: space scrolls the page, and a scroll jump
        // on a key that sometimes acts and sometimes does not is worse than
        // either behaviour consistently.
        event.preventDefault();
        if (loading) return;
        const pass = decision?.actions?.find((a) => a.kind === 'PassPriority');
        if (pass) onAct?.(pass.index, {});
      } else if (event.key === 'Escape') {
        onCancel?.();
        onDismissError?.();
      }
    }

    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  });

  /**
   * A 409 `stale_decision` means the client answered a superseded action list —
   * `api.rs` says so verbatim, and the remedy it names is "re-read GET /api/game
   * and retry". Do it rather than making the user click.
   *
   * No loop risk: a successful refresh clears `error`, and a refresh that fails
   * differently replaces the kind. A refresh swallowed by the `loading` guard
   * leaves the strip up with its retry button, which is the honest outcome.
   */
  $effect(() => {
    if (error?.kind === 'stale_decision') {
      onRefresh?.();
    }
  });

  /** Prose for the error strip, by envelope `kind` (`ApiError.kind` in `api.rs`). */
  const errorHeadline = $derived.by(() => {
    if (!error) return '';
    switch (error.kind) {
      // 422: syntactically fine, addressed to a real action, and the *engine*
      // refused it — an illegal target, an unpayable cost. The message is the
      // `GameStateError` rendered as text.
      case 'rejected':
        return 'The engine refused this play';
      case 'stale_decision':
        return 'Your view was out of date — refreshing';
      case 'no_pending_decision':
        return 'There is nothing to answer right now';
      case 'not_pregame':
        return 'Too late to mulligan — the game has begun';
      case 'no_session':
        return 'No game is running';
      case 'unknown_action':
      case 'bad_params':
      case 'invalid_body':
      case 'malformed_json':
        return 'The client sent something the server could not use';
      default:
        return 'Request failed';
    }
  });
</script>

<div class="action-bar">
  {#if error}
    <div class="error-strip" class:engine={error.kind === 'rejected'} role="alert">
      <div class="error-body">
        <span class="error-headline">{errorHeadline}</span>
        <span class="error-detail">{error.message}</span>
        {#if error.status}<span class="error-status">HTTP {error.status}</span>{/if}
      </div>
      <button class="error-dismiss" onclick={() => onDismissError?.()} title="Dismiss (Esc)">
        ✕
      </button>
    </div>
  {/if}

  {#if decision}
    <div class="bar-row">
      <div class="decision-heading">
        <span class="decision-kind">{decision.kind}</span>
        <span class="decision-seq">seq {decision.seq}</span>
      </div>

      <div class="action-groups">
        <div class="action-group plays">
          {#if plays.length === 0}
            <span class="no-plays">No plays available.</span>
          {:else}
            {#each plays as option (option.index)}
              <button
                class="action-btn kind-{option.kind}"
                class:empty-set={declaresEmptySet(option.kind)}
                disabled={loading}
                title="{option.kind}{option.needs_x
                  ? ' — needs X (Session 7)'
                  : ''}{declaresEmptySet(option.kind)
                  ? ' — submits an EMPTY set: no attacker/blocker picker exists until Session 7, and the declaration is irreversible for this combat'
                  : ''}"
                onclick={() => submit(option)}
              >
                {option.label}
                {#if option.needs_x}<span class="needs-x">X</span>{/if}
                {#if declaresEmptySet(option.kind)}<span class="empty-tag">declares none</span>{/if}
              </button>
            {/each}
          {/if}
        </div>

        {#if controls.length > 0}
          <div class="action-group controls">
            {#each controls as option (option.index)}
              <button
                class="action-btn control kind-{option.kind}"
                disabled={loading}
                title={option.kind === 'PassPriority' ? 'Pass priority (space)' : option.kind}
                onclick={() => submit(option)}
              >
                {option.label}
                {#if option.kind === 'PassPriority'}<span class="key-hint">space</span>{/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>
    </div>

    {#if passAction}
      <div class="hint">space = pass priority · Esc = cancel selection</div>
    {/if}
  {:else}
    <div class="bar-row empty">
      <span class="empty-reason">{emptyReason}</span>
      <button class="action-btn control" disabled={loading} onclick={() => onRefresh?.()}>
        Refresh
      </button>
    </div>
  {/if}
</div>

<style>
  .action-bar {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    background: #111120;
    border-top: 1px solid #2a2a44;
    padding: 0.35rem 0.6rem;
    font-family: monospace;
  }

  .bar-row {
    display: flex;
    align-items: flex-start;
    gap: 0.6rem;
  }

  .bar-row.empty {
    align-items: center;
    justify-content: space-between;
  }

  .decision-heading {
    display: flex;
    flex-direction: column;
    min-width: 8rem;
  }

  .decision-kind {
    font-size: 0.8rem;
    color: #fa0;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .decision-seq {
    font-size: 0.62rem;
    color: #445;
  }

  .action-groups {
    display: flex;
    flex: 1;
    gap: 0.6rem;
    justify-content: space-between;
    flex-wrap: wrap;
  }

  .action-group {
    display: flex;
    flex-wrap: wrap;
    gap: 0.3rem;
  }

  .action-group.controls {
    margin-left: auto;
  }

  .no-plays {
    font-size: 0.75rem;
    color: #556;
    align-self: center;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.25rem 0.5rem;
    font-size: 0.76rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
  }

  .action-btn:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .action-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .action-btn.control {
    background: #24243c;
    color: #aab;
  }

  .action-btn.kind-Concede {
    border-color: #613;
    color: #c88;
  }

  .key-hint,
  .needs-x,
  .empty-tag {
    font-size: 0.6rem;
    color: #667;
    border: 1px solid #33335a;
    border-radius: 2px;
    padding: 0 0.2rem;
  }

  .action-btn.empty-set {
    border-color: #7a5a10;
  }

  .action-btn.empty-set .empty-tag {
    color: #fc8;
    border-color: #7a5a10;
  }

  .empty-reason {
    font-size: 0.78rem;
    color: #778;
  }

  .hint {
    font-size: 0.62rem;
    color: #445;
  }

  .error-strip {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.3rem 0.45rem;
    background: #2a1420;
    border: 1px solid #6a2a40;
    border-radius: 3px;
  }

  .error-strip.engine {
    background: #2a2010;
    border-color: #7a5a10;
  }

  .error-body {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 0.4rem;
  }

  .error-headline {
    font-size: 0.76rem;
    font-weight: bold;
    color: #f9a;
  }

  .error-strip.engine .error-headline {
    color: #fc8;
  }

  .error-detail {
    font-size: 0.74rem;
    color: #ddd;
    word-break: break-word;
  }

  .error-status {
    font-size: 0.62rem;
    color: #776;
  }

  .error-dismiss {
    background: none;
    border: none;
    color: #a88;
    cursor: pointer;
    font-size: 0.8rem;
    line-height: 1;
  }

  .error-dismiss:hover {
    color: #fdd;
  }
</style>
