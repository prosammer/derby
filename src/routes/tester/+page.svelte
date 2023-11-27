<script lang="ts">
  import AudioTranscriber from '$lib/audioTranscriber';
  import { onMount } from "svelte";
  import { listen } from '@tauri-apps/api/event';
  import AudioTranscriberButton from "$components/AudioTranscriberButton.svelte";
  import ChatBubble from "$components/ChatBubble.svelte";
  import ChatInput from "$components/ChatInput.svelte";
	import { writable } from 'svelte/store';
  import type { Message } from "$lib/types/message";
  import type { Word } from "$lib/types/word";
  import { error, info } from "tauri-plugin-log-api";

  let audioTranscriber = new AudioTranscriber();
  let audioSelect: HTMLSelectElement;
  let isStreaming = false;
  let elemChat: HTMLElement;
  let currentSpeaker = 0;
  let initialMessage: Message = {
    id: 0,
    speaker: 0,
    content: '',
  };
  let messageFeed = writable<Message[]>([initialMessage]);


  const unlisten = listen('transcript', (event: any) => {
    if (event.payload && Array.isArray(event.payload.words)) {
      const words: Array<Word> = event.payload.words as Array<Word>;
      words.forEach(word => {
        info(word.speaker + ": " + word.word);
        if (word.speaker == currentSpeaker) {
          let lastMessage = $messageFeed[$messageFeed.length - 1];
          lastMessage.content += word.word + ' ';
          $messageFeed = $messageFeed;
        } else {
          addNewMessage(word);
        }
      });
    } else {
      error('event.payload.words is not an array:', event.payload);
    }
  });


  function addNewMessage(word: Word) {
    info('Adding new message');
    const newMessage = {
      id: $messageFeed.length,
      speaker: word.speaker,
      content: word.word + ' ',
    };
    $messageFeed = [...$messageFeed, newMessage];
    currentSpeaker = word.speaker;
    // Smooth scroll to bottom
    // Timeout prevents race condition
    setTimeout(() => {
      scrollChatBottom('smooth');
    }, 0);
  }

  // When DOM mounted, scroll to bottom
  onMount(() => {
    getStream().then(getDevices).then(gotDevices);
    scrollChatBottom();
  });

  async function getStream() {
    console.log('Getting stream');
    if (window.stream) {
      window.stream.getTracks().forEach(track => track.stop());
    }
    const audioSource = audioSelect.value;
    const constraints = { audio: { deviceId: audioSource ? { exact: audioSource } : undefined } };
    try {
      window.stream = await navigator.mediaDevices.getUserMedia(constraints);
    } catch (error) {
      console.error('Error: ', error);
    }
  }

  async function getDevices() {
    info('Getting devices')
    return navigator.mediaDevices.enumerateDevices();
  }

  function gotDevices(deviceInfos: MediaDeviceInfo[]) {
    info('Got devices')
    for (const deviceInfo of deviceInfos) {
      if (deviceInfo.kind === 'audioinput') {
        const option = document.createElement('option');
        option.value = deviceInfo.deviceId;
        option.text = deviceInfo.label || `Microphone ${audioSelect.options.length + 1}`;
        audioSelect.appendChild(option);
      }
    }
  }


  async function toggleStreaming() {
    if (isStreaming) {
      await audioTranscriber.stopAudioCapture();
    } else {
      const audioSource = audioSelect.value;
      const constraints = { audio: { deviceId: audioSource ? { exact: audioSource } : undefined } };
      await audioTranscriber.startAudioCapture();
    }
    isStreaming = !isStreaming;
  }


  // For some reason, eslint thinks ScrollBehavior is undefined...
  // eslint-disable-next-line no-undef
  function scrollChatBottom(behavior?: ScrollBehavior): void {
    elemChat.scrollTo({ top: elemChat.scrollHeight, behavior });
  }
</script>
<select bind:this={audioSelect} id="audioSource"></select>
<AudioTranscriberButton {isStreaming} on:toggle={() => toggleStreaming()} />

<section class="card">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[500px] p-4 overflow-y-auto space-y-4">
        {#each $messageFeed as bubble (bubble.id)}
          <p>Bubble:</p>
          <ChatBubble {bubble} />
        {/each}
      </section>
      <!-- Prompt -->
<!--      <ChatInput bind:currentMessage={currentMessage} on:send={() => onSend()} />-->
    </div>
  </div>
</section>