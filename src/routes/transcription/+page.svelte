<script lang="ts">
  import { onMount } from 'svelte';
  import HostBubble from "$components/HostBubble.svelte";
  import GuestBubble from "$components/GuestBubble.svelte";
  import type { Bubble } from "$lib/Bubble";
  import { ChatService } from "$lib/ChatService";
  import { appWindow } from "@tauri-apps/api/window";
  import LoadingDots from "$components/LoadingDots.svelte";

  let typingAnimation = true;
  let bubbleFeed: Bubble[] = [];
  let chatService: ChatService;

  onMount(() => {
    chatService = new ChatService((updatedBubbles) => {
      bubbleFeed = updatedBubbles;
      typingAnimation = false; // Stop animation when a new message is fully received
    });

    let unlistenStart: () => void;
    let unlistenChunks: () => void;

    // IIFE to handle async operations
    (async () => {
      const listeners = await chatService.setupListeners();
      unlistenStart = listeners.unlistenStart;
      unlistenChunks = listeners.unlistenChunks;
    })();

    return () => {
      // Cleanup function
      if (unlistenChunks) unlistenChunks();
      if (unlistenStart) unlistenStart();
    };
  });
</script>

<div class="bg-[#07323A]/50 p-2">
  <div data-tauri-drag-region class="flex justify-between pb-2">
    <svg on:click={() => appWindow.close()} class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="gray" >
      <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  </div>
  <div class="col-span-full text-center text-sm text-[#FDF7E3] h-64 overflow-auto">
    <div class="grid grid-cols-[auto_1fr] gap-2">
      {#each bubbleFeed as bubble}
        <div class="col-span-full">
          {#if bubble.host}
            <HostBubble {bubble} />
          {:else}
            <GuestBubble {bubble} />
          {/if}
        </div>
      {/each}
    </div>
    {#if typingAnimation}
      <LoadingDots />
    {/if}
  </div>
</div>