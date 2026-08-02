<script>
  /**
   * ValuePrompt — CR 601.2b {X} announcement and CR 700.2 mode selection.
   *
   * M11-local Session 7 (`memory/m11-session-plan.md` §4, item 6).
   *
   * Props:
   *   needsX (bool) — `ActionOptionView.needs_x`
   *   modes (ModeOptionView[]) — `{index, label, target_slots}`; empty for a
   *     non-modal action
   *   modeMin (number), modeMax (number) — CR 700.2a: how many modes must be
   *     chosen. Both 0 when `modes` is empty.
   *   disabled (bool)
   *   onConfirm (fn({x_value, modes_chosen})) — `modes_chosen` is emitted in
   *     ascending index order regardless of click order; either key is present
   *     only when this prompt actually asked for it (an action with no `{X}`
   *     never contributes `x_value`, an unmodal action never contributes
   *     `modes_chosen` — `ActionBar` merges what it gets into the running params
   *     object, and `params.rs` rejects a param the action has no channel for)
   *   onCancel (fn)
   *
   * # Announcement precedes targeting
   *
   * CR 601.2b announces the mode and any `{X}` value as part of casting; CR
   * 601.2c chooses targets afterward. `ActionBar` is the caller that enforces the
   * ordering (this component first in its picker chain), so this component does
   * not need to know about targets at all — it reports only what it collected.
   *
   * # Per-mode target slots are not rendered here
   *
   * `ModeOptionView.target_slots` exists so a later `TargetPicker` knows which
   * slots apply once modes are chosen; this component only reports which modes
   * were chosen, in ascending order, and `ActionBar` is responsible for
   * concatenating the chosen modes' `target_slots` before opening the next
   * picker in the chain.
   */
  const {
    needsX = false,
    modes = [],
    modeMin = 0,
    modeMax = 0,
    disabled = false,
    onConfirm = null,
    onCancel = null,
  } = $props();

  /** CR 601.2b: 0 is a legal default (e.g. an unpaid-optional {X}). */
  let xValue = $state(0);

  /** Selected mode indices, in click order (re-sorted ascending on confirm). */
  let chosenModes = $state([]);

  const chosenCount = $derived(chosenModes.length);
  const isModal = $derived(modes.length > 0);

  /** CR 700.2a range check. Vacuously true when this action is not modal. */
  const modesInRange = $derived(!isModal || (chosenCount >= modeMin && chosenCount <= modeMax));

  const canConfirm = $derived(modesInRange);

  function toggleMode(index) {
    if (disabled) return;
    if (chosenModes.includes(index)) {
      chosenModes = chosenModes.filter((i) => i !== index);
    } else {
      chosenModes = [...chosenModes, index];
    }
  }

  function confirm() {
    if (disabled || !canConfirm) return;
    const result = {};
    if (needsX) result.x_value = xValue;
    if (isModal) result.modes_chosen = [...chosenModes].sort((a, b) => a - b);
    onConfirm?.(result);
  }
</script>

<div class="value-prompt">
  <div class="picker-header">
    <span class="picker-title">Announce</span>
  </div>

  {#if needsX}
    <div class="x-row">
      <label for="x-value-input">X =</label>
      <input
        id="x-value-input"
        type="number"
        min="0"
        step="1"
        disabled={disabled}
        value={xValue}
        oninput={(e) => {
          const n = Number.parseInt(e.currentTarget.value, 10);
          xValue = Number.isFinite(n) && n >= 0 ? n : 0;
        }}
      />
    </div>
  {/if}

  {#if isModal}
    <div class="modes">
      <span class="modes-range">choose {modeMin === modeMax ? modeMin : `${modeMin}–${modeMax}`}</span>
      {#each modes as mode (mode.index)}
        <button
          class="mode-btn"
          class:selected={chosenModes.includes(mode.index)}
          disabled={disabled}
          onclick={() => toggleMode(mode.index)}
        >
          {mode.label}
        </button>
      {/each}
    </div>
  {/if}

  <div class="picker-actions">
    <button class="confirm" disabled={disabled || !canConfirm} onclick={confirm}>Confirm</button>
    <button class="cancel" disabled={disabled} onclick={() => onCancel?.()}>Back</button>
  </div>
</div>

<style>
  .value-prompt {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.4rem 0.6rem;
    background: #151530;
    border-top: 1px solid #2a2a44;
    font-family: monospace;
  }

  .picker-header {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .picker-title {
    font-size: 0.78rem;
    color: #adf;
    font-weight: bold;
  }

  .x-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.76rem;
    color: #ccd;
  }

  .x-row input {
    width: 4rem;
    background: #141428;
    border: 1px solid #33335a;
    border-radius: 3px;
    color: #ccd;
    font-family: monospace;
    font-size: 0.76rem;
    padding: 0.15rem 0.3rem;
  }

  .modes {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 0.3rem;
  }

  .modes-range {
    font-size: 0.68rem;
    color: #667;
    margin-right: 0.2rem;
  }

  .mode-btn {
    padding: 0.2rem 0.45rem;
    font-size: 0.74rem;
    background: #1c1c38;
    color: #ccd;
    border: 1px solid #33335a;
    border-radius: 3px;
    cursor: pointer;
    text-align: left;
  }

  .mode-btn:hover:not(:disabled) {
    background: #2a2a58;
    border-color: #4a4a90;
  }

  .mode-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .mode-btn.selected {
    background: #23386a;
    border-color: #3a5aa0;
    color: #dde;
  }

  .picker-actions {
    display: flex;
    gap: 0.4rem;
  }

  .confirm {
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    background: #23386a;
    color: #dde;
    border: 1px solid #3a5aa0;
    border-radius: 3px;
    cursor: pointer;
  }

  .confirm:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .cancel {
    padding: 0.2rem 0.5rem;
    font-size: 0.76rem;
    background: #241824;
    color: #caa;
    border: 1px solid #4a2a4a;
    border-radius: 3px;
    cursor: pointer;
  }

  .cancel:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
