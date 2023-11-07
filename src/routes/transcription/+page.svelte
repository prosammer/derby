<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let gptContent = '';

  onMount(() => {
    let unlisten: () => void;

    // Immediately invoke the async function to listen to the event
    (async () => {
      // Listen to the global `gpt-response` event
      unlisten = await listen('gpt-response', (event) => {
        if (typeof event.payload === 'string') {
          console.log(event.payload);
          gptContent = event.payload;
        } else {
          console.error('Received non-string payload', event.payload);
        }
      });
    })();

    return () => {
      // This will remove the event listener when the component is unmounted
      if (unlisten) unlisten();
    };
  });
</script>

<div data-tauri-drag-region class="grid grid-cols-12 bg-[#07323A]/50 p-2 rounded-lg">
  <div class="px-4 flex justify-center text-sm text-white">
    {gptContent}
  </div>
</div>
