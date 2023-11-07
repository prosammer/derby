<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { writeText } from '@tauri-apps/api/clipboard';
  import { appWindow } from "@tauri-apps/api/window";

  let gptContent = '';
  let hasCopied = false;
  let isNewMessage = false;

  onMount(() => {
    let unlistenChunks: () => void;
    let unlistenStart: () => void;

    (async () => {
        unlistenStart = await listen('gpt_stream_start', () => {
          isNewMessage = true; // Set the flag when a new stream starts
        });

      unlistenChunks = await listen('gpt_chunk_received', (event) => {
        if (isNewMessage) {
          gptContent = ''; // Clear the content if the previous stream was complete
          isNewMessage = false;
        }
        if (typeof event.payload === 'string') {
          // Append the new text chunks to the existing gptContent
          gptContent += event.payload;
        } else {
          console.error('Received non-string payload', event.payload);
        }
      });
    })();

    return () => {
      // Clean up the event listener when the component is unmounted
      if (unlistenChunks) unlistenChunks();
      if (unlistenStart) unlistenStart();
    };
  });

  function closeModal() {
    appWindow.close();
  }

  async function copyToClipboard() {
    try {
      await writeText(gptContent);
      hasCopied = true;
      console.log('Content copied to clipboard');

      setTimeout(() => {
        hasCopied = false;
      }, 1000);
    } catch (error) {
      console.error('Failed to copy content to clipboard', error);
      // Optionally show an error notification to the user
    }
  }
</script>

<div class="bg-[#07323A]/50 p-2">
  <div data-tauri-drag-region class="flex justify-between">
    <svg on:click={closeModal} class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="gray" >
      <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <svg on:click={copyToClipboard} class:scale-110={hasCopied} class:rotate-scale-up={hasCopied} class="w-5 h-5 cursor-pointer transition duration-150 ease-in-out"  xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="gray" >
      <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
    </svg>
  </div>
  <div class="col-span-full text-center text-sm text-[#FDF7E3] h-64 overflow-auto">
    {gptContent}
  </div>
</div>
<style>
    @keyframes rotate-scale-up {
        0% {
            transform: scale(1) rotateZ(0);
        }
        50% {
            transform: scale(2) rotateZ(140deg);
        }
        100% {
            transform: scale(1) rotateZ(0deg);
        }
    }

    .rotate-scale-up {
        animation: rotate-scale-up 0.3s ease-in-out forwards;
    }
</style>