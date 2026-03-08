<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type {
    RenameConfig,
    RenameOperation,
    FormatType,
    FormatSeparator,
    TextPosition,
  } from '$lib/types';
  import { validatePrefix, validateStartingNumber } from '$lib/types';

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

  // -------------------------------------------------------------------------
  // Validation helpers
  // -------------------------------------------------------------------------

  function validateAndDispatch() {
    // Validate prefix
    const prefixResult = validatePrefix(config.prefix);
    prefixError = prefixResult.valid ? '' : (prefixResult.error ?? '');

    // Validate starting number
    const numResult = validateStartingNumber(config.startingNumber);
    startingNumberError = numResult.valid ? '' : (numResult.error ?? '');

    // Always emit – parent can check validity via errors
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
      op = { type: 'find-replace', find: '', replace: '' };
    } else if (type === 'add-text') {
      op = { type: 'add-text', text: '', position: 'before' };
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

  function onFormatTypeChange(index: number, op: Extract<RenameOperation, { type: 'format' }>, newFormatType: string) {
    let newFormat: FormatType;
    if (newFormatType === 'sequential') {
      newFormat = { type: 'sequential', start: 1, increment: 1 };
    } else {
      newFormat = { type: 'date', format: 'YYYY-MM-DD' };
    }
    updateOperation(index, { ...op, format: newFormat });
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
          <li class="operation-item">
            <div class="operation-header">
              <span class="operation-type-badge">{op.type}</span>
              <button
                class="remove-btn"
                type="button"
                on:click={() => removeOperation(index)}
                aria-label="Remove operation {index + 1}"
              >
                ✕
              </button>
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
                    placeholder="Text to find"
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
                    <option value="date">Date stamp</option>
                  </select>
                </div>

                {#if op.format.type === 'sequential'}
                  <div class="field">
                    <label class="label" for="op-seq-start-{index}">Start</label>
                    <input
                      id="op-seq-start-{index}"
                      class="input"
                      type="number"
                      min="1"
                      max="9999"
                      value={op.format.start}
                      on:input={(e) => {
                        const val = parseInt((e.target as HTMLInputElement).value, 10);
                        updateOperation(index, { ...op, format: { ...op.format, start: isNaN(val) ? 1 : val } as FormatType });
                      }}
                    />
                  </div>
                  <div class="field">
                    <label class="label" for="op-seq-inc-{index}">Increment</label>
                    <input
                      id="op-seq-inc-{index}"
                      class="input"
                      type="number"
                      min="1"
                      value={op.format.increment}
                      on:input={(e) => {
                        const val = parseInt((e.target as HTMLInputElement).value, 10);
                        updateOperation(index, { ...op, format: { ...op.format, increment: isNaN(val) ? 1 : val } as FormatType });
                      }}
                    />
                  </div>
                {:else if op.format.type === 'date'}
                  <div class="field">
                    <label class="label" for="op-date-fmt-{index}">Date format</label>
                    <input
                      id="op-date-fmt-{index}"
                      class="input"
                      type="text"
                      value={op.format.format}
                      placeholder="YYYY-MM-DD"
                      on:input={(e) => {
                        updateOperation(index, { ...op, format: { type: 'date', format: (e.target as HTMLInputElement).value } });
                      }}
                    />
                  </div>
                {/if}

                <div class="field">
                  <label class="label" for="op-fmt-sep-{index}">Separator</label>
                  <select
                    id="op-fmt-sep-{index}"
                    class="select"
                    value={op.separator}
                    on:change={(e) => updateOperation(index, { ...op, separator: (e.target as HTMLSelectElement).value as FormatSeparator })}
                  >
                    <option value="none">None</option>
                    <option value="_">Underscore ( _ )</option>
                    <option value="-">Hyphen ( - )</option>
                  </select>
                  <p class="hint">Placed between the formatted value and the filename.</p>
                </div>

                <div class="field">
                  <label class="label" for="op-fmt-position-{index}">Position</label>
                  <select
                    id="op-fmt-position-{index}"
                    class="select"
                    value={op.position}
                    on:change={(e) => updateOperation(index, { ...op, position: (e.target as HTMLSelectElement).value as TextPosition })}
                  >
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
      <button class="add-op-btn" type="button" on:click={() => addOperation('find-replace')}>
        + Find &amp; Replace
      </button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('add-text')}>
        + Add Text
      </button>
      <button class="add-op-btn" type="button" on:click={() => addOperation('format')}>
        + Format
      </button>
    </div>
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
  }
</style>
