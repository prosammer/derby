<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();

  export let isStreaming = false;

  async function toggleStreaming() {
    dispatch('toggle');
    isStreaming = !isStreaming;
  }
</script>

<button class="btn variant-filled" on:click={toggleStreaming}>
  {isStreaming ? 'Stop' : 'Start'} Streaming
</button>