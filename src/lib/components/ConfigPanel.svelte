<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type {
    RenameConfig,
    RenameOperation,
    FormatType,
    FormatSeparator,
    TextPosition,
    CaseType,
    NumberPosition,
    AdditionalTextPosition,
  } from '$lib/types';
  import { validatePrefix, validateStartingNumber, hasNamePart } from '$lib/types';

  // Props
  export let config: RenameConfig = {
    prefix: 'BFR',
    usePrefix: true,
    useNumber: true,
    useAdditionalText: true,
    includeOriginalName: true,
    startingNumber: 1,
    additionalText: undefined,
    additionalTextPosition: 'after',
    numberPadding: 4,
    numberPosition: 'start',
    endNumberStart: 1,
    endNumberPadding: 2,
    operations: [],
    disabledSteps: [],
  };

  const dispatch = createEventDispatcher<{ configChange: RenameConfig }>();

  // Validation state
  let prefixError = '';
  let startingNumberError = '';

  // Local string binding for additionalText (to handle empty string → undefined)
  let additionalTextValue = config.additionalText ?? '';

  // Mode tabs. "template" edits the base name; the others edit rename steps.
  type Tab = 'template' | 'find-replace' | 'add-remove' | 'change-case' | 'format';
  let activeTab: Tab = 'template';

  const tabs: { id: Tab; label: string; types: RenameOperation['type'][] }[] = [
    { id: 'template', label: 'Template', types: [] },
    { id: 'find-replace', label: 'Find & Replace', types: ['find-replace'] },
    { id: 'add-remove', label: 'Add / Remove', types: ['add-text', 'remove-text'] },
    { id: 'change-case', label: 'Change Case', types: ['change-case'] },
    { id: 'format', label: 'Format', types: ['format'] },
  ];

  $: visibleOps = config.operations
    .map((op, index) => ({ op, index }))
    .filter(({ op }) => (tabs.find((t) => t.id === activeTab)?.types ?? []).includes(op.type));

  function tabCount(types: RenameOperation['type'][]): number {
    return config.operations.filter((op) => types.includes(op.type)).length;
  }

  $: nameHasPart = hasNamePart(config);

  // -------------------------------------------------------------------------
  // Validation helpers
  // -------------------------------------------------------------------------

  function validateAndDispatch() {
    const prefixResult = config.usePrefix === false ? { valid: true } : validatePrefix(config.prefix);
    prefixError = prefixResult.valid ? '' : ('error' in prefixResult ? prefixResult.error ?? '' : '');

    const numResult = config.useNumber === false ? { valid: true } : validateStartingNumber(config.startingNumber);
    startingNumberError = numResult.valid ? '' : ('error' in numResult ? numResult.error ?? '' : '');

    dispatch('configChange', { ...config });
  }

  function onPrefixInput(e: Event) {
    config.prefix = (e.target as HTMLInputElement).value;
    validateAndDispatch();
  }

  function onUsePrefixChange(e: Event) {
    config.usePrefix = (e.target as HTMLInputElement).checked;
    validateAndDispatch();
  }

  function onUseNumberChange(e: Event) {
    config.useNumber = (e.target as HTMLInputElement).checked;
    validateAndDispatch();
  }

  function onUseAdditionalTextChange(e: Event) {
    config.useAdditionalText = (e.target as HTMLInputElement).checked;
    validateAndDispatch();
  }

  function onIncludeOriginalNameChange(e: Event) {
    config.includeOriginalName = (e.target as HTMLInputElement).checked;
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

  function setTextPosition(pos: AdditionalTextPosition) {
    config.additionalTextPosition = pos;
    validateAndDispatch();
  }

  function setNumberPosition(pos: NumberPosition) {
    config.numberPosition = pos;
    validateAndDispatch();
  }

  function onNumberPaddingInput(e: Event) {
    const parsed = parseInt((e.target as HTMLInputElement).value, 10);
    config.numberPadding = isNaN(parsed) ? 4 : Math.min(10, Math.max(1, parsed));
    validateAndDispatch();
  }

  function onEndNumberStartInput(e: Event) {
    const parsed = parseInt((e.target as HTMLInputElement).value, 10);
    config.endNumberStart = isNaN(parsed) ? 1 : parsed;
    validateAndDispatch();
  }

  function onEndNumberPaddingInput(e: Event) {
    const parsed = parseInt((e.target as HTMLInputElement).value, 10);
    config.endNumberPadding = isNaN(parsed) ? 2 : parsed;
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

  function isStepOn(index: number): boolean {
    return !(config.disabledSteps ?? []).includes(index);
  }

  function setStepOn(index: number, on: boolean) {
    const off = new Set(config.disabledSteps ?? []);
    if (on) off.delete(index);
    else off.add(index);
    config.disabledSteps = [...off].sort((a, b) => a - b);
    validateAndDispatch();
  }

  let showStepMenu = false;

  function addStepAndShow(type: RenameOperation['type'], tab: Tab) {
    addOperation(type);
    showStepMenu = false;
    activeTab = tab;
  }

  function removeOperation(index: number) {
    config.operations = config.operations.filter((_, i) => i !== index);
    // Keep the "off" markers pointing at the same steps after the removal
    config.disabledSteps = (config.disabledSteps ?? [])
      .filter((i) => i !== index)
      .map((i) => (i > index ? i - 1 : i));
    validateAndDispatch();
  }

  function updateOperation(index: number, updated: RenameOperation) {
    config.operations = config.operations.map((op, i) => (i === index ? updated : op));
    validateAndDispatch();
  }

  /** Swap a step with its neighbour in the currently visible list. */
  function moveVisible(pos: number, dir: -1 | 1) {
    const other = pos + dir;
    if (other < 0 || other >= visibleOps.length) return;
    const ops = [...config.operations];
    const i = visibleOps[pos].index;
    const j = visibleOps[other].index;
    [ops[i], ops[j]] = [ops[j], ops[i]];
    config.operations = ops;
    // The "off" markers travel with their steps
    const iOff = !isStepOn(i);
    const jOff = !isStepOn(j);
    const off = new Set(config.disabledSteps ?? []);
    off.delete(i);
    off.delete(j);
    if (iOff) off.add(j);
    if (jOff) off.add(i);
    config.disabledSteps = [...off].sort((a, b) => a - b);
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
</script>

<div class="config-panel">
  <!-- Mode tabs -->
  <div class="tabs" role="tablist">
    {#each tabs as tab}
      <button
        class="tab"
        class:tab-active={activeTab === tab.id}
        type="button"
        role="tab"
        aria-selected={activeTab === tab.id}
        on:click={() => (activeTab = tab.id)}
      >
        {tab.label}{#if tab.types.length > 0 && tabCount(tab.types) > 0}<span class="tab-badge">{tabCount(tab.types)}</span>{/if}
      </button>
    {/each}
  </div>

  {#if !nameHasPart}
    <p class="error-message" role="alert">Pick at least one part of the name.</p>
  {/if}

  {#if activeTab === 'template'}
  <section class="section">
    <p class="hint">Tick the parts to include in each new name.</p>

    <div class="template-grid">
    <!-- Prefix -->
    <div class="field" class:field-off={config.usePrefix === false}>
      <label class="checkbox-label">
        <input type="checkbox" checked={config.usePrefix !== false} on:change={onUsePrefixChange} />
        Prefix
      </label>
      <input
        id="prefix"
        class="input"
        class:input-error={prefixError}
        type="text"
        value={config.prefix}
        disabled={config.usePrefix === false}
        placeholder="BFR"
        on:input={onPrefixInput}
        aria-label="Prefix"
        aria-describedby={prefixError ? 'prefix-error' : undefined}
        aria-invalid={prefixError ? 'true' : undefined}
      />
      {#if prefixError}
        <p id="prefix-error" class="error-message" role="alert">{prefixError}</p>
      {/if}
    </div>

    <!-- Number -->
    <div class="field" class:field-off={config.useNumber === false}>
      <label class="checkbox-label">
        <input type="checkbox" checked={config.useNumber !== false} on:change={onUseNumberChange} />
        {config.numberPosition === 'end' ? 'Number after prefix' : 'Sequential number'} <span class="optional">{config.numberPosition === 'end' ? '(same on every file)' : '(starting at)'}</span>
      </label>
      <input
        id="starting-number"
        class="input"
        class:input-error={startingNumberError}
        type="number"
        value={config.startingNumber}
        disabled={config.useNumber === false}
        min="1"
        max="9999"
        step="1"
        on:input={onStartingNumberInput}
        aria-label="Starting number"
        aria-describedby={startingNumberError ? 'starting-number-error' : undefined}
        aria-invalid={startingNumberError ? 'true' : undefined}
      />
      {#if startingNumberError}
        <p id="starting-number-error" class="error-message" role="alert">{startingNumberError}</p>
      {/if}
    </div>

    <!-- Additional Text -->
    <div class="field" class:field-off={config.useAdditionalText === false}>
      <label class="checkbox-label">
        <input type="checkbox" checked={config.useAdditionalText !== false} on:change={onUseAdditionalTextChange} />
        Extra text
      </label>
      <input
        id="additional-text"
        class="input"
        type="text"
        value={additionalTextValue}
        disabled={config.useAdditionalText === false}
        placeholder="e.g. DailyRush"
        on:input={onAdditionalTextInput}
        aria-label="Extra text"
      />
      <div class="segmented" role="group" aria-label="Extra text position">
        <button type="button" class="seg" class:seg-active={config.additionalTextPosition === 'before'}
          aria-pressed={config.additionalTextPosition === 'before'}
          disabled={config.useAdditionalText === false} on:click={() => setTextPosition('before')}>Before number</button>
        <button type="button" class="seg" class:seg-active={config.additionalTextPosition !== 'before'}
          aria-pressed={config.additionalTextPosition !== 'before'}
          disabled={config.useAdditionalText === false} on:click={() => setTextPosition('after')}>After number</button>
      </div>
    </div>

    </div>

    <!-- Original name -->
    <div class="field">
      <label class="checkbox-label">
        <input type="checkbox" checked={config.includeOriginalName !== false} on:change={onIncludeOriginalNameChange} />
        Keep original filename
      </label>
      <p class="hint">Untick to start blank. The file extension is always kept.</p>
    </div>

    <!-- Number placement -->
    <div class="field" class:field-off={config.useNumber === false}>
      <span class="label" id="number-position-label">Increasing number goes</span>
      <div class="segmented" role="group" aria-labelledby="number-position-label">
        <button
          type="button"
          class="seg"
          class:seg-active={(config.numberPosition ?? 'start') === 'start'}
          aria-pressed={(config.numberPosition ?? 'start') === 'start'}
          disabled={config.useNumber === false}
          on:click={() => setNumberPosition('start')}
        >At start</button>
        <button
          type="button"
          class="seg"
          class:seg-active={config.numberPosition === 'end'}
          aria-pressed={config.numberPosition === 'end'}
          disabled={config.useNumber === false}
          on:click={() => setNumberPosition('end')}
        >At end</button>
      </div>
      <p class="hint">
        {#if config.numberPosition === 'end'}
          The number after the prefix stays the same on every file. A counter that
          increases is added at the end (e.g. BFR_001_PICS_003, BFR_001_PICS_004).
        {:else}
          The number after the prefix increases on every file (e.g. BFR_001_PORTRAIT, BFR_002_PORTRAIT).
        {/if}
      </p>
    </div>

    <div class="template-grid">
      <div class="field" class:field-off={config.useNumber === false}>
        <label class="label" for="number-padding">{config.numberPosition === 'end' ? 'Fixed number digits' : 'Number digits'}</label>
        <input id="number-padding" class="input" type="number" min="1" max="10" step="1"
          value={config.numberPadding ?? 4} disabled={config.useNumber === false} on:input={onNumberPaddingInput} />
      </div>

      {#if config.numberPosition === 'end' && config.useNumber !== false}
        <div class="field">
          <label class="label" for="end-number-start">Counter starts at</label>
          <input id="end-number-start" class="input" type="number" min="0" max="9999" step="1"
            value={config.endNumberStart ?? 1} on:input={onEndNumberStartInput} />
        </div>
        <div class="field">
          <label class="label" for="end-number-padding">Counter digits</label>
          <input id="end-number-padding" class="input" type="number" min="1" max="10" step="1"
            value={config.endNumberPadding ?? 2} on:input={onEndNumberPaddingInput} />
        </div>
      {/if}
    </div>

    <!-- Add step -->
    <button class="link-btn" type="button" aria-expanded={showStepMenu} on:click={() => (showStepMenu = !showStepMenu)}>+ Add step</button>
    {#if showStepMenu}
      <div class="add-ops">
        <button class="add-op-btn" type="button" on:click={() => addStepAndShow('find-replace', 'find-replace')}>Find &amp; Replace</button>
        <button class="add-op-btn" type="button" on:click={() => addStepAndShow('add-text', 'add-remove')}>Add Text</button>
        <button class="add-op-btn" type="button" on:click={() => addStepAndShow('remove-text', 'add-remove')}>Remove Text</button>
        <button class="add-op-btn" type="button" on:click={() => addStepAndShow('change-case', 'change-case')}>Change Case</button>
        <button class="add-op-btn" type="button" on:click={() => addStepAndShow('format', 'format')}>Format</button>
      </div>
    {/if}
  </section>
  {/if}

  <!-- -----------------------------------------------------------------------
       Operation builder (Requirement 12)
  ----------------------------------------------------------------------- -->
  {#if activeTab !== 'template'}
  <section class="section">
    <p class="hint">Steps run in order on the original name, after the template is built.</p>

    {#if visibleOps.length > 0}
      <ul class="operations-list">
        {#each visibleOps as { op, index }, pos (index)}
          <li class="operation-item" class:step-off={!isStepOn(index)}>
            <div class="operation-header">
              <label class="checkbox-label step-toggle">
                <input type="checkbox" checked={isStepOn(index)} on:change={(e) => setStepOn(index, (e.target as HTMLInputElement).checked)} aria-label="Step enabled" />
                <span class="operation-type-badge">{op.type}</span>
              </label>
              <div class="op-header-actions">
                <button class="move-btn" type="button" on:click={() => moveVisible(pos, -1)} disabled={pos === 0} aria-label="Move up">↑</button>
                <button class="move-btn" type="button" on:click={() => moveVisible(pos, 1)} disabled={pos === visibleOps.length - 1} aria-label="Move down">↓</button>
                <button
                  class="remove-btn"
                  type="button"
                  on:click={() => removeOperation(index)}
                  aria-label="Remove step {pos + 1}"
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
      <p class="empty-ops">No steps yet.</p>
    {/if}

    <div class="add-ops">
      {#if activeTab === 'find-replace'}
        <button class="add-op-btn" type="button" on:click={() => addOperation('find-replace')}>+ Find &amp; Replace</button>
      {:else if activeTab === 'add-remove'}
        <button class="add-op-btn" type="button" on:click={() => addOperation('add-text')}>+ Add Text</button>
        <button class="add-op-btn" type="button" on:click={() => addOperation('remove-text')}>+ Remove Text</button>
      {:else if activeTab === 'change-case'}
        <button class="add-op-btn" type="button" on:click={() => addOperation('change-case')}>+ Change Case</button>
      {:else if activeTab === 'format'}
        <button class="add-op-btn" type="button" on:click={() => addOperation('format')}>+ Format</button>
      {/if}
    </div>
  </section>
  {/if}
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


  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.875rem;
    color: #374151;
    cursor: pointer;
  }






  .tabs {
    display: flex;
    flex-wrap: wrap;
    gap: 0.25rem;
    padding: 0.125rem;
    background-color: #f3f4f6;
    border-radius: 0.5rem;
  }

  .tab {
    flex: 1 1 auto;
    padding: 0.375rem 0.5rem;
    font-size: 0.8125rem;
    font-weight: 500;
    border: none;
    border-radius: 0.375rem;
    background: none;
    color: #374151;
    cursor: pointer;
    white-space: nowrap;
  }

  .tab-active {
    background-color: #4f46e5;
    color: #fff;
  }

  .tab-badge {
    margin-left: 0.25rem;
    padding: 0 0.375rem;
    font-size: 0.6875rem;
    border-radius: 9999px;
    background-color: rgba(255, 255, 255, 0.25);
  }

  .field-off .label,
  .field-off .optional {
    opacity: 0.5;
  }

  .input:disabled,
  .select:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .template-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(9rem, 1fr));
    gap: 0.75rem;
  }

  .step-toggle {
    gap: 0.5rem;
  }

  .step-off .operation-fields {
    opacity: 0.45;
  }

  .segmented {
    display: inline-flex;
    align-self: flex-start;
    padding: 0.125rem;
    background-color: #f3f4f6;
    border-radius: 0.5rem;
  }

  .seg {
    padding: 0.3125rem 0.875rem;
    font-size: 0.8125rem;
    font-weight: 500;
    border: none;
    border-radius: 0.375rem;
    background: none;
    color: #374151;
    cursor: pointer;
  }

  .seg:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .seg-active {
    background-color: #4f46e5;
    color: #fff;
  }

  .link-btn {
    align-self: flex-start;
    background: none;
    border: none;
    padding: 0;
    font-size: 0.8125rem;
    color: #4f46e5;
    cursor: pointer;
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
    .tabs,
    .segmented {
      background-color: #111827;
    }

    .seg {
      color: #d1d5db;
    }

    .seg-active {
      color: #fff;
    }

    .tab {
      color: #d1d5db;
    }

    .tab-active {
      background-color: #4f46e5;
      color: #fff;
    }


    .link-btn {
      color: #818cf8;
    }

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


    .checkbox-label {
      color: #d1d5db;
    }




  }
</style>
