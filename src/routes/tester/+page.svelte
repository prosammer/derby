<script lang="ts">
  import AudioTranscriber from "$lib/audioTranscriber";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import ChatBubble from "$components/ChatBubble.svelte";
  import { writable } from "svelte/store";
  import type { Message } from "$lib/types/message";
  import type { Word } from "$lib/types/word";
  import { error, info } from "tauri-plugin-log-api";
  import { Mic, Send, Disc3 } from "lucide-svelte";

  let audioTranscriber = new AudioTranscriber();
  let isStreaming = false;
  let elemChat: HTMLElement;
  let currentSpeaker = 0;

  let currentMessage: Message = {
    id: 2,
    speaker: 0,
    content: '',
    color: 'variant-soft-primary',
  };

  let messageFeed = writable<Message[]>([]);


  const unlisten = listen('transcript', (event: any) => {
    if (event.payload && Array.isArray(event.payload.words)) {
      const words: Array<Word> = event.payload.words as Array<Word>;
      words.forEach(word => {
        info(word.speaker + ": " + word.word);
        if (word.speaker == currentSpeaker) {
          currentMessage.content += word.word + ' ';
        } else {
          submitCurrentMessage();
          currentMessage = {
            id: currentMessage.id + 1,
            speaker: word.speaker,
            content: word.word + ' ',
            color: 'variant-soft-primary',
          };
        }
      });
    } else {
      error('event.payload.words is not an array:', event.payload);
    }
  });

  function createMessage(speaker: number, content = ''): Message {
    return {
      id: currentMessage.id + 1,
      speaker: speaker,
      content: content,
      color: 'variant-soft-primary',
    };
  }


  function submitCurrentMessage() {
    info('Adding new message');

    $messageFeed = [...$messageFeed, { ...currentMessage }];
    currentMessage = createMessage(currentMessage.speaker === 0 ? 1 : 0);

    // Smooth scroll to bottom
    // Timeout prevents race condition
    setTimeout(() => {
      scrollChatBottom('smooth');
    }, 0);
  }

  // When DOM mounted, scroll to bottom
  onMount(() => {
    scrollChatBottom();
  });

  async function toggleStreaming() {
    if (isStreaming) {
      await audioTranscriber.stopAudioCapture();
    } else {
      await audioTranscriber.startAudioCapture();
    }
    isStreaming = !isStreaming;
  }


  function onPromptKeydown(event: KeyboardEvent): void {
    if (['Enter'].includes(event.code)) {
      event.preventDefault();
      submitCurrentMessage();
    }
  }

  // For some reason, eslint thinks ScrollBehavior is undefined...
  // eslint-disable-next-line no-undef
  function scrollChatBottom(behavior?: ScrollBehavior): void {
    elemChat.scrollTo({ top: elemChat.scrollHeight, behavior });
  }
</script>

<section class="card opacity-90">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[400px] p-4 overflow-y-auto space-y-4">
        {#each $messageFeed as bubble}
          <ChatBubble {bubble} />
        {/each}
      </section>
      <!-- Prompt -->
      <section class="border-t border-surface-500/30 p-4">
        <div class="input-group input-group-divider grid-cols-[auto_1fr_auto] rounded-container-token">
          <button class="input-group-shim" on:click={() => toggleStreaming()}>
            {#if isStreaming}
              <Disc3 size="16"/>
            {:else}
              <Mic size="16"/>
            {/if}
          </button>
          <textarea
            bind:value={currentMessage.content}
            class="bg-transparent border-0 ring-0 text-surface-100"
            name="prompt"
            id="prompt"
            placeholder="Write a message..."
            rows="1"
            on:keydown={onPromptKeydown}
          />
          <button class="{currentMessage.content ? 'variant-filled-primary' : 'input-group-shim'}" on:click={() => submitCurrentMessage()}>
            <Send size="16"/>
          </button>
        </div>
      </section>
    </div>
  </div>
</section>