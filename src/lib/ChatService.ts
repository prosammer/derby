import type { Bubble } from "$lib/Bubble";
import { listen } from '@tauri-apps/api/event';

export class ChatService {
  private bubbleFeed: Bubble[] = [];
  private currentMessageId = 0;
  private updateCallback: (bubbles: Bubble[]) => void;

  constructor(updateCallback: (bubbles: Bubble[]) => void) {
    this.updateCallback = updateCallback;
  }

  public async setupListeners() {
    const unlistenStart = await listen('gpt_stream_start', this.handleNewMessage);
    const unlistenChunks = await listen('gpt_chunk_received', this.handleChunkReceived);
    return { unlistenStart, unlistenChunks };
  }

  private handleNewMessage = () => {
    const newMessage: Bubble = {
      id: this.currentMessageId++,
      host: false, // Adjust this based on your logic
      avatar: 14, // Set your desired avatar
      name: 'GPT-3', // Name of the sender
      timestamp: new Date().toLocaleString(),
      message: '',
      color: 'variant-soft-primary'
    };
    this.bubbleFeed = [...this.bubbleFeed, newMessage];
    this.updateCallback(this.bubbleFeed);
  };

  private handleChunkReceived = (event: { payload: string; }) => {
    const messageIndex = this.bubbleFeed.findIndex(message => message.id === this.currentMessageId - 1);
    if (messageIndex !== -1) {
      this.bubbleFeed[messageIndex].message += event.payload;
      this.updateCallback([...this.bubbleFeed]);
    }
  };
}
