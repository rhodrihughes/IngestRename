<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import type {
    RenameConfig,
    RenameOperation,
    FormatType,
    FormatSeparator,
    TextPosition,
    CaseType,
    Preset,
  } from '$lib/types';
  import { validatePrefix, validateStartingNumber } from '$lib/types';
  import { listPresets, savePreset, deletePreset } from '$lib/ipc';

  // Props
  export let config: RenameConfig = {
    prefix: 'BFR',
    startingNumber: 1,
    additionalText: undefined,
    operations: [],
  };

  const dispatch = createEventDispatcher<{ configChange: RenameConfig }>();

  // Validation state
  let prefixError = '';
  let startingNumberError = '';

  // Local string binding for additionalText (to handle empty string → undefined)
  let additionalTextValue = config.additionalText ?? '';

  // Presets
  let presets: Preset[] = [];
  let newPresetName = '';
  let presetError = '';

  onMount(async () => {
    try {
      presets = await listPresets();
    } catch { /* non-fatal */ }
  });

  // -------------------------------------------------------------------------
  // Validation helpers
  // -------------------------------------------------------------------------

  function validateAndDispatch() {
    const prefixResult = validatePrefix(config.prefix);
    prefixError = prefixResult.valid ? '' : (prefixResult.error ?? '');

    const numResult = validateStartingNumber(config.startingNumber);
    startingNumberError = numResult.valid ? '' : (numResult.error ?? '');

    dispatch('configChange', { ...config });
  }

  function onPrefixInput(e: Event) {
    config.prefix = (e.target as HTMLInputElement).value;
    validateAndDispatch();
  }

  function onStartingNumberInput(e: Event) {
    const raw = (e.target as HTMLInputElement).value;
    const parsed = parseInt(raw, 10);
    config.startingNumber = isNaN(parsed) ? 0 : parsed;
    validateAndDispatch();
  }

  function onAdditionalTextInput(e: Event) {
    additionalTextValue = (e.target as HTMLInputElement).value;
    config.additionalText = additionalTextValue.trim() === '' ? undefined : additionalTextValue;
    validateAndDispatch();
  }

  // -------------------------------------------------------------------------
  // Operation builder helpers
  // -------------------------------------------------------------------------

  function addOperation(type: RenameOperation['type']) {
    let op: RenameOperation;
    if (type === 'find-replace') {
      op = { type: 'find-replace', find: '', replace: '', useRegex: false };
    } else if (type === 'add-text') {
      op = { type: 'add-text', text: '', position: 'before' };
    } else if (type === 'change-case') {
      op = { type: 'change-case', case: 'lowercase' };
    } else if (type === 'remove-text') {
      op = { type: 'remove-text', text: '' };
    } else {
      op = { type: 'format', format: { type: 'sequential', start: 1, increment: 1 }, position: 'before', separator: 'none' };
    }
    config.operations = [...config.operations, op];
    validateAndDispatch();
  }

  function removeOperation(index: number) {
    config.operations = config.operations.filter((_, i) => i !== index);
    validateAndDispatch();
  }

  function updateOperation(index: number, updated: RenameOperation) {
    config.operations = config.operations.map((op, i) => (i === index ? updated : op));
    validateAndDispatch();
  }

  function moveOperation(from: number, to: number) {
    if (to < 0 || to >= config.operations.length) return;
    const ops = [...config.operations];
    const [item] = ops.splice(from, 1);
    ops.splice(to, 0, item);
    config.operations = ops;
    validateAndDispatch();
  }

  function onFormatTypeChange(index: number, op: Extract<RenameOperation, { type: 'format' }>, newFormatType: string) {
    let newFormat: FormatType;
    if (newFormatType === 'sequential') {
      newFormat = { type: 'sequential', start: 1, increment: 1 };
    } else if (newFormatType === 'counter') {
      newFormat = { type: 'counter', start: 1, increment: 1, padding: 4 };
    } else {
      newFormat = { type: 'date', format: 'YYYY-MM-DD' };
    }
    updateOperation(index, { ...op, format: newFormat });
  }

  // -------------------------------------------------------------------------
  // Drag-to-reorder
  // -------------------------------------------------------------------------

  let dragIndex: number | null = null;
  let dragOverIndex: number | null = null;

  function onDragStart(index: number) {
    dragIndex = index;
  }

  function onDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    dragOverIndex = index;
  }

  function onDrop(index: number) {
    if (dragIndex !== null && dragIndex !== index) {
      moveOperation(dragIndex, index);
    }
    dragIndex = null;
    dragOverIndex = null;
  }

  function onDragEnd() {
    dragIndex = null;
    dragOverIndex = null;
  }

  // -------------------------------------------------------------------------
  // Presets
  // -------------------------------------------------------------------------

  async function onSavePreset() {
    const name = newPresetName.trim();
    if (!name) { presetError = 'Enter a preset name'; return; }
    presetError = '';
    try {
      await savePreset({ name, config: { ...config } });
      presets = await listPresets();
      newPresetName = '';
    } catch (e) {
      presetError = e instanceof Error ? e.message : 'Failed to save preset';
    }
  }

  async function onLoadPreset(preset: Preset) {
    config = { ...preset.config };
    additionalTextValue = config.additionalText ?? '';
    validateAndDispatch();
  }

  async function onDeletePreset(name: string) {
    try {
      await deletePreset(name);
      presets = await listPresets();
    } catch { /* non-fatal */ }
  }
</script>

<div class="config-panel">
  <!-- -----------------------------------------------------------------------
       Basic configuration
  ----------------------------------------------------------------------- -->
  <section class="section">
    <h2 class="section-title">Rename Configuration</h2>

    <!-- Prefix -->
    <div class="field">
      <label class="label" for="prefix">Prefix</label>
      <input
        id="prefix"
        class="input"
        class:input-error={prefixError}
        type="text"
        value={config.prefix}
        placeholder="BFR"
        on:input={onPrefixInput}
        aria-describedby={prefixError ? 'prefix-error' : undefined}
        aria-invalid={prefixError ? 'true' : undefined}
      />
      {#if prefixError}
        <p id="prefix-error" class="error-message" role="alert">{prefixError}</p>
      {/if}
    </div>

    <!-- Starting Number -->
    <div class="field">
      <label class="label" for="starting-number">Starting Number</label>
      <input
        id="starting-number"
        class="input"
        class:input-error={startingNumberError}
        type="number"
        value={config.startingNumber}
        min="1"
        max="9999"
        step="1"
        on:input={onStartingNumberInput}
        aria-describedby={startingNumberError ? 'starting-number-error' : undefined}
        aria-invalid={startingNumberError ? 'true' : undefined}
      />
      {#if startingNumberError}
        <p id="starting-number-error" class="error-message" role="alert">{startingNumberError}</p>
      {/if}
    </div>

    <!-- Additional Text -->
    <div class="field">
      <label class="label" for="additional-text">Additional Text <span class="optional">(optional)</span></label>
      <input
        id="additional-text"
        class="input"
        type="text"
        value={additionalTextValue}
        placeholder="e.g. DailyRush"
        on:input={onAdditionalTextInput}
      />
      <p class="hint">Inserted between the sequential number and the original filename.</p>
    </div>
  </section>

  <!-- -----------------------------------------------------------------------
       Operation builder (Requirement 12)
  ----------------------------------------------------------------------- -->
  <section class="section">
    <h2 class="section-title">Rename Operations</h2>
    <p class="hint">Add optional operations applied to each filename.</p>

    {#if config.operations.length > 0}
      <ul class="operations-list">
        {#each config.operations as op, index (index)}
          <li
            class="operation-item"
            class:drag-over={dragOverIndex === index}
            draggable="true"
            on:dragstart={() => onDragStart(index)}
            on:dragover={(e) => onDragOver(e, index)}
            on:drop={() => onDrop(index)}
            on:dragend={onDragEnd}
          >
            <div class="operation-header">
              <span class="drag-handle" aria-hidden="true">⠿</span>
              <span class="operation-type-badge">{op.type}</span>
              <div class="op-header-actions">
                <button class="move-btn" type="button" on:click={() => moveOperation(index, index - 1)} disabled={index === 0} aria-label="Move up">↑</button>
                <button class="move-btn" type="button" on:click={() => moveOperation(index, index + 1)} disabled={index === config.operations.length - 1} aria-label="Move down">↓</button>
                <button
                  class="remove-btn"
                  type="button"
                  on:click={() => removeOperation(index)}
                  aria-label="Remove operation {index + 1}"
                >✕</button>
              </div>
            </div>

            <!-- Find & Replace -->
            {#if op.type === 'find-replace'}
              <div class="operation-fields">
                <div class="field">
                  <label class="label" for="op-find-{index}">Find</label>
                  <input
                    id="op-find-{index}"
                    class="input"
                    type="text"
                    value={op.find}
                    placeholder={op.useRegex ? 'Regex pattern' : 'Text to find'}
                    on:input={(e) => updateOperation(index, { ...op, find: (e.target as HTMLInputElement).value })}
                  />
                </div>
                <div class="field">
                  <label class="label" for="op-replace-{index}">Replace with</label>
                  <input
                    id="op-replace-{index}"
                    class="input"
                    type="text"
                    value={op.replace}
                    placeholder="Replacement text"
                    on:input={(e) => updateOperation(index, { ...op, replace: (e.target as HTMLInputElement).value })}
                  />
                </div>
                <label class="checkbox-label">
                  <input
                    type="checkbox"
                    checked={op.useRegex ?? false}
                    on:change={(e) => updateOperation(index, { ...op, useRegex: (e.target as HTMLInputElement).checked })}
                  />
                  Use regex
                </label>
              </div>

            <!-- Add Text -->
            {:else if op.type === 'add-text'}
              <div class="operation-fields">
                <div class="field">
                  <label class="label" for="op-text-{index}">Text</label>
                  <input
                    id="op-text-{index}"
                    class="input"
                    type="text"
                    value={op.text}
                    placeholder="Text to add"
                    on:input={(e) => updateOperation(index, { ...op, text: (e.target as HTMLInputElement).value })}
                  />
                </div>
                <div class="field">
                  <label class="label" for="op-position-{index}">Position</label>
                  <select
                    id="op-position-{index}"
                    class="select"
                    value={op.position}
                    on:change={(e) => updateOperation(index, { ...op, position: (e.target as HTMLSelectElement).value as TextPosition })}
                  >
                    <option value="before">Before filename</option>
                    <option value="after">After filename (before extension)</option>
                  </select>
                </div>
              </div>

            <!-- Change Case -->
            {:else if op.type === 'change-case'}
              <div class="operation-fields">
                <div class="field">
                  <label class="label" for="op-case-{index}">Case</label>
                  <select
                    id="op-case-{index}"
                    class="select"
                    value={op.case}
                    on:change={(e) => updateOperation(index, { ...op, case: (e.target as HTMLSelectElement).value as CaseType })}
                  >
                    <option value="lowercase">lowercase</option>
                    <option value="uppercase">UPPERCASE</option>
                    <option value="title-case">Title Case</option>
                    <option value="camel-case">camelCase</option>
                    <option value="snake-case">snake_case</option>
                  </select>
                </div>
              </div>

            <!-- Remove Text -->
            {:else if op.type === 'remove-text'}
              <div class="operation-fields">
                <div class="field">
                  <label class="label" for="op-remove-{index}">Text to remove</label>
                  <input
                    id="op-remove-{index}"
                    class="input"
                    type="text"
                    value={op.text}
                    placeholder="Text to remove"
                    on:input={(e) => updateOperation(index, { ...op, text: (e.target as HTMLInputElement).value })}
                  />
                </div>
              </div>

            <!-- Format -->
            {:else if op.type === 'format'}
              <div class="operation-fields">
                <div class="field">
                  <label class="label" for="op-format-type-{index}">Format type</label>
                  <select
                    id="op-format-type-{index}"
                    class="select"
                    value={op.format.type}
                    on:change={(e) => onFormatTypeChange(index, op, (e.target as HTMLSelectElement).value)}
                  >
                    <option value="sequential">Sequential number</option>
                    <option value="counter">Counter (custom padding)</option>
                    <option value="date">Date stamp</option>
                  </select>
                </div>

                {#if op.format.type === 'sequential'}
                  <div class="field">
                    <label class="label" for="op-seq-start-{index}">Start</label>
                    <input id="op-seq-start-{index}" class="input" type="number" min="1" max="9999" value={op.format.start}
                      on:input={(e) => { const val = parseInt((e.target as HTMLInputElement).value, 10); updateOperation(index, { ...op, format: { ...op.format, start: isNaN(val) ? 1 : val } as FormatType }); }} />
                  </div>
                  <div class="field">
                    <label class="label" for="op-seq-inc-{index}">Increment</label>
                    <input id="op-seq-inc-{index}" class="input" type="number" min="1" value={op.format.increment}
                      on:input={(e) => { const val = parseInt((e.target as HTMLInputElement).value, 10); updateOperation(index, { ...op, format: { ...op.format, increment: isNaN(val) ? 1 : val } as FormatType }); }} />
                  </div>
                {:else if op.format.type === 'counter'}
                  <div class="field">
                    <label class="label" for="op-ctr-start-{index}">Start</label>
                    <input id="op-ctr-start-{index}" class="input" type="number" min="1" value={op.format.start}
                      on:input={(e) => { const val = parseInt((e.target as HTMLInputElement).value, 10); updateOperation(index, { ...op, format: { ...op.format, start: isNaN(val) ? 1 : val } as FormatType }); }} />
                  </div>
                  <div class="field">
                    <label class="label" for="op-ctr-inc-{index}">Increment</label>
                    <input id="op-ctr-inc-{index}" class="input" type="number" min="1" value={op.format.increment}
                      on:input={(e) => { const val = parseInt((e.target as HTMLInputElement).value, 10); updateOperation(index, { ...op, format: { ...op.format, increment: isNaN(val) ? 1 : val } as FormatType }); }} />
                  </div>
                  <div class="field">
                    <label class="label" for="op-ctr-pad-{index}">Zero-padding digits</label>
                    <input id="op-ctr-pad-{index}" class="input" type="number" min="1" max="10" value={op.format.padding}
                      on:input={(e) => { const val = parseInt((e.target as HTMLInputElement).value, 10); updateOperation(index, { ...op, format: { ...op.format, padding: isNaN(val) ? 4 : val } as FormatType }); }} />
                  </div>
                {:else if op.format.type === 'date'}
                  <div class="field">
                    <label class="label" for="op-date-fmt-{index}">Date format</label>
                    <input id="op-date-fmt-{index}" class="input" type="text" value={op.format.format} placeholder="YYYY-MM-DD"
                      on:input={(e) => { updateOperation(index, { ...op, format: { type: 'date', format: (e.target as HTMLInputElement).value } }); }} />
                    <p class="field-hint">Tokens: YYYY, YY, MM, DD, HH, mm, ss</p>
                  </div>
                {/if}

                <div class="field">
                  <label class="label" for="op-fmt-sep-{index}">Separator</label>
                  <select id="op-fmt-sep-{index}" class="select" value={op.separator}
                    on:change={(e) => updateOperation(index, { ...op, separator: (e.target as HTMLSelectElement).value as FormatSeparator })}>
                    <option value="none">None</option>
                    <option value="_">Underscore ( _ )</option>
                    <option value="-">Hyphen ( - )</option>
                  </select>
                </div>

                <div class="field">
                  <label class="label" for="op-fmt-position-{index}">Position</label>
                  <select id="op-fmt-position-{index}" class="select" value={op.position}
                    on:change={(e) => updateOperation(index, { ...op, position: (e.target as HTMLSelectElement).value as TextPosition })}>
                    <option value="before">Before filename</option>
                    <option value="after">After filename (before extension)</option>
                  </select>
                </div>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {:else}
      <p class="empty-ops">No operations added. Use the buttons below to add operations.</p>
    {/if}

    <!-- Add operation buttons -->
    <div class="add-ops">
      <button class="add-op-btn" type="button" on:click={() => addOperation('find-replace')}>+ Find &amp; Replace</button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('add-text')}>+ Add Text</button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('change-case')}>+ Change Case</button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('remove-text')}>+ Remove Text</button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('format')}>+ Format</button>
    </div>
  </section>

  <!-- -----------------------------------------------------------------------
       Presets
  ----------------------------------------------------------------------- -->
  <section class="section">
    <h2 class="section-title">Presets</h2>

    {#if presets.length > 0}
      <ul class="presets-list">
        {#each presets as preset}
          <li class="preset-item">
            <span class="preset-name">{preset.name}</span>
            <div class="preset-actions">
              <button class="preset-load-btn" type="button" on:click={() => onLoadPreset(preset)}>Load</button>
              <button class="preset-delete-btn" type="button" on:click={() => onDeletePreset(preset.name)} aria-label="Delete preset {preset.name}">✕</button>
            </div>
          </li>
        {/each}
      </ul>
    {:else}
      <p class="hint">No saved presets.</p>
    {/if}

    <div class="preset-save-row">
      <input
        class="input"
        type="text"
        placeholder="Preset name"
        bind:value={newPresetName}
        on:keydown={(e) => e.key === 'Enter' && onSavePreset()}
      />
      <button class="add-op-btn" type="button" on:click={onSavePreset}>Save</button>
    </div>
    {#if presetError}
      <p class="error-message" role="alert">{presetError}</p>
    {/if}
  </section>
</div>

<style>
  .config-panel {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  .section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .section-title {
    font-size: 0.9375rem;
    font-weight: 600;
    color: #111827;
    margin: 0 0 0.25rem;
  }

  .field {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .label {
    font-size: 0.875rem;
    font-weight: 500;
    color: #374151;
  }

  .optional {
    font-weight: 400;
    color: #6b7280;
    font-size: 0.8125rem;
  }

  .input,
  .select {
    width: 100%;
    padding: 0.5rem 0.75rem;
    font-size: 0.875rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #111827;
    outline: none;
    transition: border-color 0.15s, box-shadow 0.15s;
    box-sizing: border-box;
  }

  .input:focus,
  .select:focus {
    border-color: #6366f1;
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.15);
  }

  .input-error {
    border-color: #ef4444;
  }

  .input-error:focus {
    border-color: #ef4444;
    box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
  }

  .field-hint {
    margin: 0;
    font-size: 0.75rem;
    color: #6b7280;
  }

  .error-message {
    font-size: 0.8125rem;
    color: #dc2626;
    margin: 0;
  }

  .hint {
    font-size: 0.8125rem;
    color: #6b7280;
    margin: 0;
  }

  /* Operations list */
  .operations-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }

  .operation-item {
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    padding: 0.75rem;
    background-color: #f9fafb;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .operation-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .drag-handle {
    cursor: grab;
    color: #9ca3af;
    font-size: 1rem;
    padding: 0 0.25rem;
    user-select: none;
  }

  .op-header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .move-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #9ca3af;
    font-size: 0.875rem;
    padding: 0.125rem 0.25rem;
    border-radius: 0.25rem;
    line-height: 1;
    transition: color 0.15s, background-color 0.15s;
  }

  .move-btn:disabled {
    opacity: 0.3;
    cursor: default;
  }

  .move-btn:not(:disabled):hover {
    color: #374151;
    background-color: #e5e7eb;
  }

  .drag-over {
    outline: 2px dashed #6366f1;
    outline-offset: 2px;
  }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: #374151;
    cursor: pointer;
  }

  .presets-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .preset-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.375rem 0.625rem;
    border: 1px solid #e5e7eb;
    border-radius: 0.375rem;
    background-color: #f9fafb;
  }

  .preset-name {
    font-size: 0.875rem;
    color: #374151;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .preset-actions {
    display: flex;
    gap: 0.25rem;
    flex-shrink: 0;
  }

  .preset-load-btn {
    padding: 0.25rem 0.5rem;
    font-size: 0.75rem;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 0.25rem;
    background-color: #fff;
    color: #374151;
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .preset-load-btn:hover {
    background-color: #f3f4f6;
  }

  .preset-delete-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #9ca3af;
    font-size: 0.75rem;
    padding: 0.25rem;
    border-radius: 0.25rem;
    transition: color 0.15s, background-color 0.15s;
  }

  .preset-delete-btn:hover {
    color: #ef4444;
    background-color: #fee2e2;
  }

  .preset-save-row {
    display: flex;
    gap: 0.5rem;
  }

  .preset-save-row .input {
    flex: 1;
  }

  .operation-type-badge {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: #4f46e5;
    background-color: #eef2ff;
    padding: 0.125rem 0.5rem;
    border-radius: 9999px;
  }

  .remove-btn {
    background: none;
    border: none;
    cursor: pointer;
    color: #9ca3af;
    font-size: 0.875rem;
    padding: 0.25rem;
    border-radius: 0.25rem;
    line-height: 1;
    transition: color 0.15s, background-color 0.15s;
  }

  .remove-btn:hover {
    color: #ef4444;
    background-color: #fee2e2;
  }

  .operation-fields {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .empty-ops {
    font-size: 0.875rem;
    color: #6b7280;
    padding: 1rem;
    text-align: center;
    border: 1px dashed #d1d5db;
    border-radius: 0.375rem;
    margin: 0;
  }

  .add-ops {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
  }

  .add-op-btn {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #374151;
    cursor: pointer;
    transition: background-color 0.15s, border-color 0.15s;
  }

  .add-op-btn:hover {
    background-color: #f3f4f6;
    border-color: #9ca3af;
  }

  /* Dark mode */
  @media (prefers-color-scheme: dark) {
    .section-title {
      color: #f9fafb;
    }

    .label {
      color: #d1d5db;
    }

    .input,
    .select {
      background-color: #1f2937;
      border-color: #374151;
      color: #f9fafb;
    }

    .input:focus,
    .select:focus {
      border-color: #818cf8;
      box-shadow: 0 0 0 3px rgba(129, 140, 248, 0.2);
    }

    .operation-item {
      background-color: #1f2937;
      border-color: #374151;
    }

    .operation-type-badge {
      background-color: #312e81;
      color: #a5b4fc;
    }

    .add-op-btn {
      background-color: #1f2937;
      border-color: #374151;
      color: #d1d5db;
    }

    .add-op-btn:hover {
      background-color: #374151;
      border-color: #4b5563;
    }

    .empty-ops {
      border-color: #374151;
      color: #9ca3af;
    }

    .hint {
      color: #9ca3af;
    }

    .optional {
      color: #9ca3af;
    }

    .drag-handle {
      color: #6b7280;
    }

    .checkbox-label {
      color: #d1d5db;
    }

    .preset-item {
      background-color: #111827;
      border-color: #374151;
    }

    .preset-name {
      color: #d1d5db;
    }

    .preset-load-btn {
      background-color: #1f2937;
      border-color: #374151;
      color: #d1d5db;
    }

    .preset-load-btn:hover {
      background-color: #374151;
    }
  }
</style>
