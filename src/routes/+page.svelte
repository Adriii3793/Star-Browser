<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { platform as osPlatform } from '@tauri-apps/plugin-os';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { LogicalSize } from '@tauri-apps/api/dpi';
  import Setup from '$lib/comp/setup/setup.svelte';
  import SetupProgress from '$lib/comp/setup/SetupProgress.svelte';
  import BroswerShell from '$lib/comp/browser/BroswerShell.svelte';
  import WindowControls from '$lib/comp/browser/WindowControls.svelte';
  import { windowChrome } from '$lib/stores/windowChrome.svelte';
  import { theme } from '$lib/stores/theme.svelte';
  import { PRESET_THEMES, type Theme } from '$lib/stores/theme.svelte';
  import { setup, DOT_COUNT } from '$lib/stores/setup.svelte';
  import { isSetupComplete } from '$lib/services/setup';
  import '../app.css';

  theme.init();

  function applySavedTheme() {
    if (setup.data.theme === 'custom' && setup.data.customBg && setup.data.customSurface && setup.data.customAccent) {
      const custom: Theme = {
        id: 'custom',
        name: 'Custom',
        bg: setup.data.customBg,
        surface: setup.data.customSurface,
        accent: setup.data.customAccent,
        image: setup.data.background
      };
      theme.set(custom);
      return;
    }
    theme.set(PRESET_THEMES.some((item) => item.id === setup.data.theme) || setup.data.theme === 'system'
      ? setup.data.theme
      : 'light');
  }

  let setupDone = $state<boolean | null>(null);
  let os = $state<'macos' | 'windows' | 'linux'>(
    typeof navigator === 'undefined' ? 'windows' : detectOs()
  );
  let squared = $derived(windowChrome.squared);
  let showSetupChrome = $derived(setupDone === false && setup.dotIndex >= 0);

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

  async function enterSetupWindowMode() {
    const win = getCurrentWindow();
    try {
      await win.unmaximize().catch(() => {});
      await win.setMaximizable(false);
      await win.setResizable(false);
      await win.setSize(new LogicalSize(880, 640));
      await win.center();
    } catch {
    }
  }

  async function exitSetupWindowMode() {
    const win = getCurrentWindow();
    try {
      await win.setResizable(true);
      await win.setMaximizable(true);
    } catch {
    }
  }

  function completeSetup() {
    setupDone = true;
    void exitSetupWindowMode();
  }

  function detectOs(): 'macos' | 'windows' | 'linux' {
    try {
      const p = osPlatform();
      return p === 'macos' ? 'macos' : p === 'linux' ? 'linux' : 'windows';
    } catch {
      const ua = navigator.userAgent;
      if (/Macintosh|Mac OS X/.test(ua)) return 'macos';
      if (/Linux|X11/.test(ua) && !/Android/.test(ua)) return 'linux';
      return 'windows';
    }
  }

  onMount(async () => {
    os = detectOs();

    try {
      setupDone = await isSetupComplete();
    } catch (e) {
      console.warn('is_setup_complete is not available', e);
      setupDone = true;
    }
    await setup.load();
    applySavedTheme();
    if (setupDone === false) await enterSetupWindowMode();
  });
</script>

<div
  class="app"
  class:rounded={!squared}
  class:setup={setupDone !== true}
>
  {#if setupDone === null || setupDone === false}
    {#snippet backButton()}
      <button
        class="back"
        type="button"
        aria-label="Go back"
        title="Go back"
        disabled={!setup.canGoBack}
        onclick={() => setup.back()}
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M19 12H5M12 19l-7-7 7-7" />
        </svg>
      </button>
    {/snippet}

    <div class="titlebar">
      {#if os === 'macos'}
        <WindowControls platform={os} maximizable={false} />
        {#if showSetupChrome}{@render backButton()}{/if}
        <div class="drag-region" data-tauri-drag-region role="presentation"></div>
      {:else}
        {#if showSetupChrome}{@render backButton()}{/if}
        <div class="drag-region" data-tauri-drag-region role="presentation"></div>
      {/if}

      {#if showSetupChrome}
        <div class="progress-slot">
          <SetupProgress count={DOT_COUNT} index={setup.dotIndex} />
        </div>
      {/if}

      {#if os !== 'macos'}
        <WindowControls platform={os} maximizable={false} />
      {/if}
    </div>

    <main class="content">
      {#if setupDone === null}
        <p class="loading">Loading</p>
      {:else}
        <Setup oncomplete={completeSetup} />
      {/if}
    </main>
  {:else}
    <BroswerShell />
  {/if}

  {#if !squared && setupDone === true}
    <button type="button" tabindex="-1" aria-label="Resize from top" class="rz rz-n" onmousedown={(e) => startResize('North', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from bottom" class="rz rz-s" onmousedown={(e) => startResize('South', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from left" class="rz rz-w" onmousedown={(e) => startResize('West', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from right" class="rz rz-e" onmousedown={(e) => startResize('East', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from top left" class="rz rz-nw" onmousedown={(e) => startResize('NorthWest', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from top right" class="rz rz-ne" onmousedown={(e) => startResize('NorthEast', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from bottom left" class="rz rz-sw" onmousedown={(e) => startResize('SouthWest', e)}></button>
    <button type="button" tabindex="-1" aria-label="Resize from bottom right" class="rz rz-se" onmousedown={(e) => startResize('SouthEast', e)}></button>
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
    --win-radius: 0px;
    position: relative;
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
    background: var(--bg-chrome);
    border-radius: var(--win-radius);
  }

  .app.rounded {
    --win-radius: 12px;
    --win-edge: 1px;
  }

  .app.rounded::after {
    content: '';
    position: absolute;
    inset: 0;
    z-index: 100000;
    border: 1px solid var(--border-strong);
    border-top: none;
    border-radius: var(--win-radius);
    pointer-events: none;
  }

  .app.setup {
    background: var(--bg-page);
  }

  .titlebar {
    display: flex;
    align-items: stretch;
    height: 38px;
    flex-shrink: 0;
    background: transparent;
    user-select: none;
    position: relative;
    z-index: 10001;
  }

  .drag-region {
    flex: 1 1 auto;
    min-width: 72px;
    align-self: stretch;
    -webkit-app-region: drag;
  }

  .back {
    -webkit-app-region: no-drag;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 0 0 auto;
    align-self: center;
    width: 28px;
    height: 28px;
    margin-left: 8px;
    padding: 0;
    border: none;
    border-radius: 50%;
    background: transparent;
    color: var(--text-soft);
    cursor: pointer;
    transition:
      background-color 0.15s ease,
      color 0.15s ease,
      opacity 0.15s ease;
  }

  .back:hover:not(:disabled) {
    background: var(--hover);
    color: var(--text);
  }

  .back:disabled {
    opacity: 0;
    pointer-events: none;
  }

  .back:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .back svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 2.2;
    stroke-linecap: round;
    stroke-linejoin: round;
  }

  .progress-slot {
    position: absolute;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    pointer-events: none;
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
