<script lang="ts">
  import AudioTranscriber from '$lib/audioTranscriber';
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event';

  let audioTranscriber = new AudioTranscriber();
  let isStreaming = false;


  async function toggleStreaming() {
    if (isStreaming) {
      await audioTranscriber.stopAudioCapture();
    } else {
      await audioTranscriber.startAudioCapture();
    }
    isStreaming = !isStreaming;
  }

  interface TranscriptEventPayload {
    bot: boolean;
    transcript: string;
  }

  const unlisten = listen('transcript', (event) => {
    const { bot, transcript } = event.payload as TranscriptEventPayload;
    addMessage(bot, transcript);
  });

  interface MessageFeed {
    id: number;
    bot: boolean;
    name: string;
    message: string;
    color: string;
  }

  let elemChat: HTMLElement;

  // Messages
  let messageFeed: MessageFeed[] = [];
  let currentMessage = '';

  // For some reason, eslint thinks ScrollBehavior is undefined...
  // eslint-disable-next-line no-undef
  function scrollChatBottom(behavior?: ScrollBehavior): void {
    elemChat.scrollTo({ top: elemChat.scrollHeight, behavior });
  }

  function getCurrentTimestamp(): string {
    return new Date().toLocaleString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true });
  }

  function addMessage(speaker: boolean, transcript: string): void {
    const newMessage = {
      id: messageFeed.length,
      bot: speaker,
      name: 'Jane',
      timestamp: `Today @ ${getCurrentTimestamp()}`,
      message: transcript,
      color: 'variant-soft-primary'
    };
    // Update the message feed
    messageFeed = [...messageFeed, newMessage];
    // Clear prompt
    currentMessage = '';
    // Smooth scroll to bottom
    // Timeout prevents race condition
    setTimeout(() => {
      scrollChatBottom('smooth');
    }, 0);
  }

  function onPromptKeydown(event: KeyboardEvent): void {
    if (['Enter'].includes(event.code)) {
      event.preventDefault();
      // addMessage();
    }
  }

  // When DOM mounted, scroll to bottom
  onMount(() => {
    scrollChatBottom();
  });
</script>
<button class="btn variant-filled" on:click={() => toggleStreaming()}>
  {isStreaming ? 'Stop' : 'Start'} Streaming
</button>
<section class="card">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[500px] p-4 overflow-y-auto space-y-4">
        {#each messageFeed as bubble}
          {#if bubble.bot === true}
            <div class="grid grid-cols-[auto_1fr] gap-2">
              <div class="card p-4 variant-soft rounded-tl-none space-y-2">
                <header class="flex justify-between items-center">
                  <p class="font-bold">{bubble.name}</p>
                </header>
                <p>{bubble.message}</p>
              </div>
            </div>
          {:else}
            <div class="grid grid-cols-[1fr_auto] gap-2">
              <div class="card p-4 rounded-tr-none space-y-2 {bubble.color}">
                <header class="flex justify-between items-center">
                  <p class="font-bold">{bubble.name}</p>
                </header>
                <p>{bubble.message}</p>
              </div>
            </div>
          {/if}
        {/each}
      </section>
      <!-- Prompt -->
      <section class="border-t border-surface-500/30 p-4">
        <div class="input-group input-group-divider grid-cols-[auto_1fr_auto] rounded-container-token">
          <button class="input-group-shim">+</button>
          <textarea
            bind:value={currentMessage}
            class="bg-transparent border-0 ring-0"
            name="prompt"
            id="prompt"
            placeholder="Write a message..."
            rows="1"
            on:keydown={onPromptKeydown}
          />
          <button class={currentMessage ? 'variant-filled-primary' : 'input-group-shim'}}>
            <i class="fa-solid fa-paper-plane" />
          </button>
        </div>
      </section>
    </div>
  </div>
</section>