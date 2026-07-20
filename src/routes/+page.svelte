<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { platform as osPlatform } from '@tauri-apps/plugin-os';
  import Setup from '$lib/comp/setup/setup.svelte';
  import BroswerShell from '$lib/comp/broswer/BroswerShell.svelte';
  import WindowControls from '$lib/comp/broswer/WindowControls.svelte';
  import '../app.css';

  let setupDone = $state<boolean | null>(null);
  let os = $state<'macos' | 'windows' | 'linux'>('windows');

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
      console.warn('is_setup_complete non disponibile', e);
      setupDone = true;
    }
  });
</script>

<div class="app">
  {#if setupDone === null || setupDone === false}
    <div class="titlebar" class:mac={os === 'macos'}>
      {#if os === 'macos'}
        <WindowControls platform={os} />
        <div class="tabs">
          <span class="title" data-tauri-drag-region>star</span>
        </div>
        <div class="drag-region" data-tauri-drag-region></div>
      {:else}
        <div class="tabs">
          <span class="title" data-tauri-drag-region>star</span>
        </div>
        <div class="drag-region" data-tauri-drag-region></div>
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
</div>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    height: 100%;
    overflow: hidden;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
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
</style>