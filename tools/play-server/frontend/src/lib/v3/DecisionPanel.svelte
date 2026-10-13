<script>
  /**
   * DecisionPanel — the dock's decision slot and the centred decision box
   * (`docs/play-surface-v3-brief.md` §1 rule 11).
   *
   * It hosts exactly ONE `PromptPanel`, mounted for the life of the page:
   * `PromptPanel` owns the picker chain, the space / Escape keys and the
   * auto-open of a lone blocking answer, and all three would reset if it were
   * unmounted when the box moves. So the box does not move in the DOM at all —
   * the same `.decision-host` wrapper gains the mockup's `.decision-float`
   * class, whose `position: fixed` lifts it to the centre of the screen.
   *
   * On a `Priority` decision the answers that are cards (a land to play, a
   * spell to cast, an ability to activate, a mana source) are drawn as cards
   * with a caption, grouped as in the mockup. They are the complement of what
   * `PromptPanel` already draws as buttons: its `looseActions` covers every
   * action whose `object_id` is NOT in `renderedIds`, and the parent passes
   * `renderedIds` = the keys of `cardIndex`. On any other decision kind the
   * picker `PromptPanel` opens is the answer and no groups are shown.
   *
   * A card item is also where a targeting arrow starts (C2): a press on it that
   * moves becomes a drag (`targeting.svelte.js`), a press that does not is the
   * same click as before. While a target stage is open the panel says, in one
   * line, that the table itself can be clicked — or that it cannot.
   */
  import PromptPanel from '../v2/PromptPanel.svelte';
  import CardFace from './CardFace.svelte';
  import BlockUi from './BlockUi.svelte';
  import { answerable } from '../v2/prompt.js';
  import { splitCamel } from '../names.js';
  import { beginArrowFromAction, targeting } from './targeting.svelte.js';

  const {
    cardIndex = new Map(),
    decision = null,
    loading = false,
    error = null,
    passUntil = null,
    humanName = null,
    turn = null,
    gameOver = null,
    pregame = false,
    chooser = null,
    clickMessage = null,
    hasGame = false,
    renderedIds = null,
    onAct = null,
    onRefresh = null,
    onDismissError = null,
    onPassUntil = null,
    onCancelPassUntil = null,
    onDismissPassUntil = null,
    onClientError = null,
    onKeep = null,
    onMulligan = null,
    onNewGame = null,
    onExport = null,
    onChooseOption = null,
    onClearSelection = null,
    onChainOpenChange = null,
  } = $props();

  let panel = $state(null);
  let minimised = $state(false);
  let chainOpen = $state(false);

  export function beginExternal(option) {
    panel?.beginExternal(option);
  }

  const over = $derived(!!gameOver);
  const answering = $derived(!!decision || pregame || over);
  const autoPassing = $derived(!!passUntil && passUntil.stopReason === null);
  // Never centred during an auto-pass run: a decision arrives and is answered
  // several times a second, and the box would strobe.
  const floating = $derived(answering && !minimised && !autoPassing);
  const title = $derived(over ? 'Game over' : decision || pregame ? 'Your decision' : 'Waiting');

  // A new priority re-opens the box; so does the game starting or ending.
  let lastSeq;
  $effect(() => {
    const seq = decision?.seq;
    if (seq === undefined || seq === lastSeq) return;
    lastSeq = seq;
    minimised = false;
  });
  $effect(() => {
    if (pregame || over) minimised = false;
  });

  function handleChainOpenChange(open) {
    chainOpen = open;
    // A picker needs the room: one opened from a board click while the box is
    // in the corner brings the box back to the centre.
    if (open) minimised = false;
    onChainOpenChange?.(open);
  }

  // ── Card items ─────────────────────────────────────────────────────────────

  const CAST_KINDS = new Set(['PlayLand', 'CastSpell', 'CastWithMutate', 'CastMorphFaceDown']);
  const ACTIVATE_KINDS = new Set(['ActivateAbility', 'ActivateLoyaltyAbility', 'ActivateBloodrush', 'SaddleMount']);
  const KIND_WORD = {
    PlayLand: 'play land',
    CastSpell: 'cast',
    ActivateAbility: 'activate',
    ActivateLoyaltyAbility: 'loyalty',
    TapForMana: 'mana',
  };

  /** The server's label minus the card's own name and the leading verb. */
  function caption(action, name) {
    let rest = action.label ?? '';
    if (name) rest = rest.split(name).join(' ');
    rest = rest
      .replace(/^\s*(Cast|Play|Activate|Tap)\b/i, '')
      .replace(/\s+/g, ' ')
      .trim()
      .replace(/\s+of$/i, '')
      .replace(/^[\s—:·-]+|[\s—:·-]+$/g, '');
    return rest || KIND_WORD[action.kind] || splitCamel(action.kind).toLowerCase();
  }

  const groups = $derived.by(() => {
    if (decision?.kind !== 'Priority') return [];
    const cast = [];
    const activate = [];
    const other = [];
    const mana = new Map();
    for (const a of decision.actions ?? []) {
      if (a.kind === 'PassPriority' || a.kind === 'Concede') continue;
      if (!answerable(a)) continue;
      if (a.object_id === null || a.object_id === undefined) continue;
      const card = cardIndex?.get(Number(a.object_id));
      if (!card) continue;
      const item = { action: a, card, cap: caption(a, card.name), count: 0 };
      if (a.kind === 'TapForMana') {
        // Identical sources (five Forests) show once; the click taps the first.
        const key = `${card.name}\u0000${a.label}`;
        const seen = mana.get(key);
        if (seen) seen.count = (seen.count || 1) + 1;
        else mana.set(key, item);
      } else if (CAST_KINDS.has(a.kind)) cast.push(item);
      else if (ACTIVATE_KINDS.has(a.kind)) activate.push(item);
      else other.push(item);
    }
    return [
      { title: 'Play / cast', muted: false, items: cast },
      { title: 'Activate', muted: false, items: activate },
      { title: 'Other', muted: false, items: other },
      { title: 'Mana', muted: true, items: [...mana.values()] },
    ].filter((g) => g.items.length > 0);
  });

  const moreOptions = $derived(groups.filter((g) => !g.muted).reduce((n, g) => n + g.items.length, 0));

  function choose(item) {
    if (loading) return;
    panel?.beginExternal(item.action);
  }
</script>

<div class="prompt-box">
  <div class="panel-title">{title}</div>
  {#if floating}
    <div class="dim">open in the centre of the table</div>
  {/if}
  <div class="decision-host" class:decision-float={floating}>
    {#if floating}
      <div class="head">
        <div class="panel-title">{title}</div>
        <button class="mini-btn" onclick={() => (minimised = true)}>▁ minimise to the corner</button>
      </div>
    {/if}

    <PromptPanel
      bind:this={panel}
      {decision}
      {loading}
      {error}
      {passUntil}
      {humanName}
      {turn}
      {gameOver}
      {pregame}
      {chooser}
      {clickMessage}
      {hasGame}
      {renderedIds}
      {onAct}
      {onRefresh}
      {onDismissError}
      {onPassUntil}
      {onCancelPassUntil}
      {onDismissPassUntil}
      {onClientError}
      {onKeep}
      {onMulligan}
      {onNewGame}
      {onExport}
      {onChooseOption}
      {onClearSelection}
      onChainOpenChange={handleChainOpenChange}
    />

    <!-- Arrow targeting (C2): the picker above stays the list; this says the table is live too. -->
    {#if targeting.live}
      <div class="arrow-hint">
        Or choose on the table: click a highlighted card or player{targeting.live.objectId !== null ? ', or drag from the source card' : ''}.
      </div>
    {:else if targeting.unavailable}
      <div class="arrow-hint off">Arrows are not available for this choice — use the list.</div>
    {/if}

    <!-- Blocker arrows (C2): draws itself only while `BlockerPicker` is mounted in this host. -->
    <BlockUi {decision} {cardIndex} {loading} />

    {#if floating && !chainOpen}
      {#each groups as group (group.title)}
        <div class="group" class:muted={group.muted}>
          <div class="gtitle">{group.title}</div>
          <div class="items">
            {#each group.items as item (item.action.index)}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div
                class="act"
                class:disabled={loading}
                title={item.action.label}
                data-action-index={item.action.index}
                onpointerdown={(event) => beginArrowFromAction(item.action, event)}
                onclick={() => choose(item)}
              >
                <CardFace card={item.card} size="perm" count={item.count} lit targetable={false} />
                <div class="cap">{item.cap}</div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    {/if}

    {#if !floating && minimised && answering && !autoPassing}
      <button class="mini-btn" onclick={() => (minimised = false)}>
        ⤢ open in the centre{moreOptions > 0 ? ` · ${moreOptions} more option${moreOptions === 1 ? '' : 's'}` : ''}
      </button>
    {/if}
  </div>
</div>
