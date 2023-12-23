<script lang="ts">
  import AudioTranscriber from "$lib/audioTranscriber";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import ChatBubble from "$components/ChatBubble.svelte";
  import type { Word } from "$lib/types/word";
  import { Mic, Send, Disc3 } from "lucide-svelte";
  import { appWindow, LogicalSize } from "@tauri-apps/api/window";
  import { readEnvVariable } from "$lib/utils";
  import OpenAI from 'openai';
  import { writable } from "svelte/store";

  let OPENAI_API_KEY: string;
  let DEEPGRAM_API_KEY: string;

  interface Message {
    id: string;
    content: string;
    ui?: string;
    role: 'system' | 'user' | 'assistant' | 'function';
    name?: string;
    function_call?: string;
  }


  let openai: OpenAI;
  let input = writable('');
  let initialMessage: Message =  {
    id: '0',
    content: '',
    role: 'user'
  }
  let messages = writable<Message[]>([initialMessage]);
  let audioTranscriber: AudioTranscriber;
  let isStreaming = false;
  let elemChat: HTMLElement;

  $: if($messages && $messages.length > 0) {
    resizeWindowToFitMessages();
  }

  // When DOM mounted, scroll to bottom
  onMount(async () => {
    scrollChatBottom();
    OPENAI_API_KEY = await readEnvVariable("OPENAI_API_KEY");
    DEEPGRAM_API_KEY = await readEnvVariable('DEEPGRAM_API_KEY');
    audioTranscriber = new AudioTranscriber(DEEPGRAM_API_KEY);
    openai = new OpenAI({
      apiKey: OPENAI_API_KEY,
      dangerouslyAllowBrowser: true
    })
    // input, handleSubmit, messages = useChat();
    await processTranscript();
  });

  async function resizeWindowToFitMessages() {
    const newHeight =  400;
    const currentSize = await appWindow.innerSize();
    const factor = await appWindow.scaleFactor();
    const logicalSize = currentSize.toLogical(factor);
    await appWindow.setSize(new LogicalSize(logicalSize.width, newHeight));
  }

  function processTranscript() {
    return listen('transcript', (event: any) => {
      if (event.payload && Array.isArray(event.payload.words)) {
        const words: Array<Word> = event.payload.words as Array<Word>;
        words.forEach(word => {
          input.update(value => value + ' ' + word.word);
        });
      }
    })
  }

  async function handleSubmit() {
    let newMessage: Message = {
      id: $messages.length.toString(),
      content: $input,
      role: "user",
    }

    $messages.push(newMessage);
    input.set("");

    const stream = openai.beta.chat.completions.stream({
      model: 'gpt-4',
      messages: $messages.map(message => ({
        role: message.role,
        content: message.content
      })),
      stream: true,
    });

    let responseMessage: Message = {
      id: $messages.length.toString(),
      content: '',
      role: 'assistant'
    }
    $messages.push(responseMessage);

    stream.on('content', (delta, snapshot) => {
      $messages[$messages.length - 1].content += delta;
    });


  }

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

  function handleInputEvent(event: { key: any }) {
    if (event.key == "Enter") {
      handleSubmit();
    }
  }
</script>

<section class="card opacity-90">
  <div class="chat w-full h-full grid grid-cols-1 lg:grid-cols-[30%_1fr]">
    <!-- Chat -->
    <div class="grid grid-row-[1fr_auto]">
      <!-- Conversation -->
      <section bind:this={elemChat} class="max-h-[400px] p-4 overflow-y-auto space-y-4">
        {#if $messages}
        {#each $messages as message}
          <ChatBubble {message} />
        {/each}
        {/if}
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
          <input
            type="text"
            bind:value={$input}
            class="bg-transparent border-0 ring-0 text-surface-100"
            name="prompt"
            id="prompt"
            placeholder="Write a message..."
            on:keydown={handleInputEvent}
          />
          <button class="{$input ? 'variant-filled-primary' : 'input-group-shim'}"  on:click={() => handleSubmit()}>
            <Send size="16"/>
          </button>
        </div>
      </section>
    </div>
  </div>
</section>