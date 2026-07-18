<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Setup from '$lib/comp/setup/setup.svelte';
  import BroswerShell from '$lib/comp/broswer/BroswerShell.svelte';
  import WindowControls from '$lib/comp/broswer/WindowControls.svelte';
  let setupDone = $state<boolean | null>(null);

  onMount(async () => {
    try {
      setupDone = await invoke('is_setup_complete');
    } catch (e) {
      console.warn('is_setup_complete  non', e);
      setupDone = true;
    }
  });
  import '../app.css'
</script>


{#if setupDone === null}
  <p style="padding: 1rem;">Loading</p>
  {:else if setupDone === false}
    <Setup />
  {:else}
    <BroswerShell />
{/if}