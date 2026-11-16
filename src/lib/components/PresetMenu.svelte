<script lang="ts">
  import { onMount, createEventDispatcher } from 'svelte';
  import type { Preset, RenameConfig } from '$lib/types';
  import { createDefaultConfig } from '$lib/types';
  import { listPresets, savePreset, deletePreset, getSettings, setDefaultPreset } from '$lib/ipc';

  /** Current configuration, saved when the user presses Save. */
  export let config: RenameConfig;

  /** Name of the preset currently applied, if any. Can be set by the parent (e.g. at startup). */
  export let activeName: string | null = null;

  // Serialized config when the active preset was applied, used to flag edits
  let snapshot: string | null = null;
  $: if (activeName && snapshot === null) snapshot = JSON.stringify(config);
  $: if (!activeName) snapshot = null;
  $: modified = !!activeName && snapshot !== null && JSON.stringify(config) !== snapshot;

  const dispatch = createEventDispatcher<{ load: RenameConfig }>();

  let open = false;
  let presets: Preset[] = [];
  let defaultName: string | null = null;
  let newName = '';
  let error = '';

  onMount(async () => {
    try {
      presets = await listPresets();
    } catch { /* non-fatal */ }
    try {
      defaultName = (await getSettings()).defaultPreset ?? null;
    } catch { /* non-fatal */ }
  });

  function load(preset: Preset) {
    const loaded = { ...createDefaultConfig(), ...preset.config };
    activeName = preset.name;
    snapshot = JSON.stringify(loaded);
    dispatch('load', loaded);
    open = false;
  }

  async function save() {
    const name = newName.trim();
    if (!name) { error = 'Enter a preset name'; return; }
    error = '';
    try {
      await savePreset({ name, config: { ...config } });
      presets = await listPresets();
      activeName = name;
      snapshot = JSON.stringify(config);
      newName = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to save preset';
    }
  }

  async function toggleDefault(name: string) {
    const next = defaultName === name ? null : name;
    try {
      await setDefaultPreset(next);
      defaultName = next;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to set default preset';
    }
  }

  async function remove(name: string) {
    try {
      await deletePreset(name);
      if (defaultName === name) defaultName = null;
      if (activeName === name) activeName = null;
      presets = await listPresets();
    } catch { /* non-fatal */ }
  }

  function onWindowClick(e: MouseEvent) {
    if (open && !(e.target as HTMLElement).closest('.preset-menu')) open = false;
  }
</script>

<svelte:window on:click={onWindowClick} />

<div class="preset-menu">
  <button class="trigger" type="button" aria-haspopup="menu" aria-expanded={open} on:click={() => (open = !open)}>
    {#if activeName}
      Preset: <span class="active-name" title={modified ? `${activeName} (modified)` : activeName}>{activeName}{modified ? ' •' : ''}</span>
    {:else}
      Presets
    {/if}
    ▾
  </button>

  {#if open}
    <div class="popover" role="menu">
      {#if presets.length === 0}
        <p class="empty">No saved presets yet.</p>
      {:else}
        <ul class="list">
          {#each presets as preset (preset.name)}
            <li class="item">
              <button class="name" type="button" role="menuitem" on:click={() => load(preset)} title="Load preset">
                {preset.name}
              </button>
              <button
                class="icon"
                class:icon-on={defaultName === preset.name}
                type="button"
                title={defaultName === preset.name ? 'Starts with this preset (click to unset)' : 'Start with this preset'}
                aria-label="Toggle {preset.name} as startup preset"
                on:click={() => toggleDefault(preset.name)}
              >★</button>
              <button class="icon icon-del" type="button" title="Delete preset" aria-label="Delete preset {preset.name}" on:click={() => remove(preset.name)}>✕</button>
            </li>
          {/each}
        </ul>
      {/if}

      <div class="save-row">
        <input
          class="input"
          type="text"
          placeholder="Save current settings as…"
          bind:value={newName}
          on:keydown={(e) => e.key === 'Enter' && save()}
        />
        <button class="save" type="button" on:click={save}>Save</button>
      </div>
      {#if error}
        <p class="error" role="alert">{error}</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .preset-menu {
    position: relative;
  }

  .trigger {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #374151;
    cursor: pointer;
  }

  .active-name {
    display: inline-block;
    max-width: 9rem;
    vertical-align: bottom;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
    color: #4f46e5;
  }

  .popover {
    position: absolute;
    right: 0;
    top: calc(100% + 0.25rem);
    z-index: 30;
    width: 18rem;
    padding: 0.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    background-color: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 0.5rem;
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.14);
  }

  .list {
    list-style: none;
    margin: 0;
    padding: 0;
    max-height: 14rem;
    overflow-y: auto;
  }

  .item {
    display: flex;
    align-items: center;
    gap: 0.125rem;
  }

  .name {
    flex: 1;
    min-width: 0;
    padding: 0.375rem 0.5rem;
    text-align: left;
    font-size: 0.8125rem;
    background: none;
    border: none;
    border-radius: 0.25rem;
    color: #111827;
    cursor: pointer;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name:hover {
    background-color: #f3f4f6;
  }

  .icon {
    background: none;
    border: none;
    padding: 0.25rem 0.375rem;
    font-size: 0.8125rem;
    color: #d1d5db;
    cursor: pointer;
    border-radius: 0.25rem;
  }

  .icon-on {
    color: #f59e0b;
  }

  .icon-del:hover {
    color: #ef4444;
    background-color: #fee2e2;
  }

  .empty {
    margin: 0;
    padding: 0.25rem 0.5rem;
    font-size: 0.8125rem;
    color: #6b7280;
  }

  .save-row {
    display: flex;
    gap: 0.375rem;
  }

  .input {
    flex: 1;
    min-width: 0;
    padding: 0.375rem 0.5rem;
    font-size: 0.8125rem;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #111827;
  }

  .save {
    padding: 0.375rem 0.75rem;
    font-size: 0.8125rem;
    font-weight: 500;
    border: 1px solid #d1d5db;
    border-radius: 0.375rem;
    background-color: #fff;
    color: #374151;
    cursor: pointer;
  }

  .error {
    margin: 0;
    font-size: 0.75rem;
    color: #dc2626;
  }

  @media (prefers-color-scheme: dark) {
    .active-name {
      color: #a5b4fc;
    }

    .trigger,
    .save,
    .input {
      background-color: #1f2937;
      border-color: #374151;
      color: #d1d5db;
    }

    .popover {
      background-color: #1f2937;
      border-color: #374151;
    }

    .name {
      color: #f3f4f6;
    }

    .name:hover {
      background-color: #374151;
    }

    .icon {
      color: #4b5563;
    }

    .icon-on {
      color: #f59e0b;
    }

    .empty {
      color: #9ca3af;
    }
  }
</style>
