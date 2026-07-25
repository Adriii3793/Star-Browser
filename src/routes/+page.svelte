<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { platform as osPlatform } from '@tauri-apps/plugin-os';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import Setup from '$lib/comp/setup/setup.svelte';
  import BroswerShell from '$lib/comp/broswer/BroswerShell.svelte';
  import WindowControls from '$lib/comp/broswer/WindowControls.svelte';
  import { windowChrome } from '$lib/stores/windowChrome.svelte';
  import '../app.css';

  let setupDone = $state<boolean | null>(null);
  let os = $state<'macos' | 'windows' | 'linux'>('windows');
  let squared = $derived(windowChrome.maximized);

  type ResizeDir =
    | 'North' | 'South' | 'East' | 'West'
    | 'NorthEast' | 'NorthWest' | 'SouthEast' | 'SouthWest';

  function startResize(direction: ResizeDir, e: MouseEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    invoke('plugin:window|start_resize_dragging', {
      label: getCurrentWindow().label,
      value: direction
    });
  }

  function toggleMaximizeOnDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    windowChrome.toggle();
  }

  onMount(async () => {
    try {
      const p = osPlatform();
      os = p === 'macos' ? 'macos' : p === 'linux' ? 'linux' : 'windows';
    } catch {
      os = 'windows';
    }

    try {
      setupDone = await invoke('is_setup_complete');
    } catch (e) {
      console.warn('is_setup_complete is not available', e);
      setupDone = true;
    }
  });
</script>

<div class="app" class:rounded={(os === 'macos' || os === 'windows') && !squared}>
  {#if setupDone === null || setupDone === false}
    <div class="titlebar" class:mac={os === 'macos'}>
      {#if os === 'macos'}
        <WindowControls platform={os} />
        <div class="tabs">
          <span class="title" data-tauri-drag-region>star</span>
        </div>
        <div class="drag-region" data-tauri-drag-region role="presentation" ondblclick={toggleMaximizeOnDrag}></div>
      {:else}
        <div class="tabs">
          <span class="title" data-tauri-drag-region>star</span>
        </div>
        <div class="drag-region" data-tauri-drag-region role="presentation" ondblclick={toggleMaximizeOnDrag}></div>
        <WindowControls platform={os} />
      {/if}
    </div>

    <main class="content">
      {#if setupDone === null}
        <p class="loading">Loading</p>
      {:else}
        <Setup />
      {/if}
    </main>
  {:else}
    <BroswerShell />
  {/if}

  {#if !squared}
    <button type="button" tabindex="-1" aria-label="Ridimensiona dall'alto" class="rz rz-n" onmousedown={(e) => startResize('North', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona dal basso" class="rz rz-s" onmousedown={(e) => startResize('South', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona da sinistra" class="rz rz-w" onmousedown={(e) => startResize('West', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona da destra" class="rz rz-e" onmousedown={(e) => startResize('East', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona dall'angolo in alto a sinistra" class="rz rz-nw" onmousedown={(e) => startResize('NorthWest', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona dall'angolo in alto a destra" class="rz rz-ne" onmousedown={(e) => startResize('NorthEast', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona dall'angolo in basso a sinistra" class="rz rz-sw" onmousedown={(e) => startResize('SouthWest', e)}></button>
    <button type="button" tabindex="-1" aria-label="Ridimensiona dall'angolo in basso a destra" class="rz rz-se" onmousedown={(e) => startResize('SouthEast', e)}></button>
  {/if}
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
    background: transparent;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-chrome);
  }

  .app.rounded {
    border-radius: 12px;
  }

  .titlebar {
    display: flex;
    align-items: stretch;
    height: 40px;
    flex-shrink: 0;
    background: #f7f5f2;
    border-bottom: 1px solid #e8e4de;
    user-select: none;
  }

  .tabs {
    display: flex;
    align-items: center;
    align-self: stretch;
    flex: 0 1 auto;
    min-width: 0;
    overflow: hidden;
  }

  .drag-region {
    flex: 1 1 auto;
    min-width: 72px;
    align-self: stretch;
    -webkit-app-region: drag;
  }

  .titlebar .title {
    padding-left: 12px;
    font-size: 13px;
    font-weight: 500;
    color: #321e1e;
  }

  .titlebar.mac .title {
    padding-left: 0;
  }

  .content {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .loading {
    padding: 1rem;
  }

  .rz {
    position: fixed;
    margin: 0;
    padding: 0;
    border: none;
    background: transparent;
    outline: none;
    z-index: 9999;
    -webkit-app-region: no-drag;
  }

  .rz-n { top: 0; left: 12px; right: 12px; height: 8px; cursor: ns-resize; }
  .rz-s { bottom: 0; left: 12px; right: 12px; height: 8px; cursor: ns-resize; }
  .rz-w { top: 12px; bottom: 12px; left: 0; width: 8px; cursor: ew-resize; }
  .rz-e { top: 12px; bottom: 12px; right: 0; width: 8px; cursor: ew-resize; }

  .rz-nw { top: 0; left: 0; width: 14px; height: 14px; cursor: nwse-resize; z-index: 10000; }
  .rz-ne { top: 0; right: 0; width: 14px; height: 14px; cursor: nesw-resize; z-index: 10000; }
  .rz-sw { bottom: 0; left: 0; width: 14px; height: 14px; cursor: nesw-resize; z-index: 10000; }
  .rz-se { bottom: 0; right: 0; width: 14px; height: 14px; cursor: nwse-resize; z-index: 10000; }
</style>