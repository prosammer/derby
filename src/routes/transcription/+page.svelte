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

<div class="grid grid-cols-12 bg-gray-300 p-2 rounded-lg">
  <!-- Small left column for the left button -->
  <div class="col-span-2 flex justify-center">
    <button class="flex items-center justify-center h-10 w-10 bg-gray-400 rounded-full text-white">
      <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
      </svg>
    </button>
  </div>

  <!-- Large central column that takes up most of the width -->
  <div class="col-span-8 flex justify-center">
    {gptContent}
  </div>

  <!-- Small right column for the right button -->
  <div class="col-span-2 flex justify-center">
    <button id="stop-recording" class="flex items-center justify-center h-10 w-10 bg-red-500 rounded-full">
      <!-- Icon or content for the button -->
    </button>
  </div>
</div>
