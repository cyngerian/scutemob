<script>
  /**
   * StackBox — the dock's stack (`docs/play-surface-v3-brief.md` §1 rule 10):
   * top item first and lit, saying it resolves next.
   *
   * The wire is bottom-first (the last element is the top of the stack), so the
   * list is reversed here. Names and targets are the seat view's own strings: a
   * face-down source arrives already blanked and is shown as sent.
   */
  const { stack = [] } = $props();

  const topFirst = $derived([...(stack ?? [])].reverse());

  /** `StackItemView.kind` → the words after the controller's name. */
  function kindWords(kind) {
    switch (kind) {
      case 'spell': return 'cast';
      case 'activated_ability': return 'activated';
      case 'triggered_ability': return 'trigger';
      default: return String(kind ?? '').replaceAll('_', ' ');
    }
  }
</script>

<div class="stack-box">
  <div class="panel-title">Stack</div>
  {#if topFirst.length === 0}
    <div class="dim">empty</div>
  {:else}
    {#each topFirst as item, i (item.id)}
      <div class="stack-item" class:top={i === 0}>
        <div class="s-who">{item.controller} {kindWords(item.kind)}{i === 0 ? ' · resolves next' : ''}</div>
        <div class="s-name">{item.source_name}</div>
        {#if item.is_copy || item.targets?.length}
          <div class="s-text">
            {#if item.is_copy}copy{/if}{#if item.is_copy && item.targets?.length}{' · '}{/if}{#if item.targets?.length}→ {item.targets.join(', ')}{/if}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>
