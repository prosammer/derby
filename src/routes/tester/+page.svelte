<script lang="ts">
  import AudioTranscriber from '$lib/audioTranscriber';
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event';
  import AudioTranscriberButton from "$components/AudioTranscriberButton.svelte";
  import ChatBubble from "$components/ChatBubble.svelte";
  import ChatInput from "$components/ChatInput.svelte";

  let audioTranscriber = new AudioTranscriber();
  let isStreaming = false;
  let elemChat: HTMLElement;
  let currentSpeaker = 0;
  let initialMessage: Message = {
    id: 0,
    speaker: 0,
    content: '',
  };
  let messageFeed: Message[] = [initialMessage];


  interface Word {
    word: string;
    start: number;
    end: number;
    confidence: number;
    speaker: number;
  }

  interface Message {
    id: number;
    speaker: number;
    content: string;
  }

  const unlisten = listen('transcript', (event: any) => {
    if (event.payload && Array.isArray(event.payload.words)) {
      const words: Array<Word> = event.payload.words as Array<Word>;
      words.forEach(word => {
        console.log(word.word);
        if (word.speaker == currentSpeaker) {
          let lastMessage = messageFeed[messageFeed.length - 1];
          lastMessage.content += word.word + ' ';
          messageFeed = [...messageFeed.slice(0, messageFeed.length - 1), lastMessage];
        } else {
          addNewMessage(word);
        }
      });
    } else {
      console.error('event.payload.words is not an array:', event.payload);
    }
  });


  function addNewMessage(word: Word) {
    console.log('Adding new message');
    const newMessage = {
      id: messageFeed.length,
      speaker: word.speaker,
      content: word.word + ' ',
    };
    messageFeed = [...messageFeed, newMessage];
    currentSpeaker = word.speaker;
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


  // For some reason, eslint thinks ScrollBehavior is undefined...
  // eslint-disable-next-line no-undef
  function scrollChatBottom(behavior?: ScrollBehavior): void {
    elemChat.scrollTo({ top: elemChat.scrollHeight, behavior });
  }

  function getCurrentTimestamp(): string {
    return new Date().toLocaleString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true });
  }
</script>
<AudioTranscriberButton {isStreaming} on:toggle={() => toggleStreaming()} />

<section class="card">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[500px] p-4 overflow-y-auto space-y-4">
        {#each messageFeed as bubble (bubble.id)}
          <p>Bubble:</p>
          <ChatBubble {bubble} />
        {/each}
      </section>
      <!-- Prompt -->
<!--      <ChatInput bind:currentMessage={currentMessage} on:send={() => onSend()} />-->
    </div>
  </div>
</section>