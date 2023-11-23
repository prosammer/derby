<script lang="ts">
  import AudioTranscriber from '$lib/audioTranscriber';
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event';
  import AudioTranscriberButton from "$components/AudioTranscriberButton.svelte";
  import ChatBubble from "$components/ChatBubble.svelte";
  import ChatInput from "$components/ChatInput.svelte";

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

  function onSend() {
    addMessage(false, "dummy text");
  }

  // When DOM mounted, scroll to bottom
  onMount(() => {
    scrollChatBottom();
  });
</script>
<AudioTranscriberButton {isStreaming} on:toggle={() => toggleStreaming()} />

<section class="card">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[500px] p-4 overflow-y-auto space-y-4">
        {#each messageFeed as bubble (bubble.id)}
          <ChatBubble {bubble} />
        {/each}
      </section>
      <!-- Prompt -->
      <ChatInput bind:currentMessage={currentMessage} on:send={() => onSend()} />
    </div>
  </div>
</section>