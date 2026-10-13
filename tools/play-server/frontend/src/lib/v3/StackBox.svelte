<script>
  /**
   * StackBox — the dock's stack (`docs/play-surface-v3-brief.md` §1 rule 10):
   * top item first and lit, saying it resolves next.
   *
   * The wire is bottom-first (the last element is the top of the stack), so the
   * list is reversed here. Names and targets are the seat view's own strings: a
   * face-down source arrives already blanked and is shown as sent.
   *
   * C3 (`scutemob-268`): each item carries `data-stack-id` (its `StackItemView.id`)
   * and `data-source-oid`, is outlined red when another item on the stack targets
   * it (an object ref naming its source card, or a `stack_object` ref naming it),
   * and — hovered or focused — asks `Arrows.svelte` for one red arrow to each of
   * its own targets that is on screen (`stackTargets.svelte.js`).
   *
   * While a target stage is open (or an arrow is being dragged) whose candidates
   * include a `stack_object`, that item carries `.target-ok` and can be clicked or
   * dropped on; a chosen one carries `.target-chosen`. The click itself is taken
   * by `targeting.svelte.js` at the table root, which presses the mounted
   * picker's own button for that candidate — nothing is submitted from here.
   */
  import { stackTargets, stackHover } from './stackTargets.svelte.js';
  import { targeting } from './targeting.svelte.js';

  const { stack = [] } = $props();

  const isTargeted = (item) =>
    stackTargets.stackItems.has(Number(item.id)) ||
    (item.source_object_id !== null && item.source_object_id !== undefined && stackTargets.objects.has(Number(item.source_object_id)));
  const refCount = (item) => stackTargets.byItem.get(Number(item.id))?.length ?? 0;
  const leave = (item) => {
    if (stackHover.id === Number(item.id)) stackHover.id = null;
  };

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
      <!-- svelte-ignore a11y_no_noninteractive_tabindex -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="stack-item"
        class:top={i === 0}
        class:targeted={isTargeted(item)}
        class:has-targets={refCount(item) > 0}
        class:target-ok={targeting.okStack.has(Number(item.id))}
        class:target-chosen={targeting.chosenStack.has(Number(item.id))}
        data-stack-id={item.id}
        data-source-oid={item.source_object_id ?? undefined}
        tabindex="0"
        onmouseenter={() => (stackHover.id = Number(item.id))}
        onmouseleave={() => leave(item)}
        onfocus={() => (stackHover.id = Number(item.id))}
        onblur={() => leave(item)}
      >
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
