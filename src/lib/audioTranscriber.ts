import { createClient, LiveTranscriptionEvents } from "@deepgram/sdk";
import { emit } from '@tauri-apps/api/event';
import { info } from "tauri-plugin-log-api";

export default class AudioTranscriber {
  private deepgram: any;
  private dgConnection: any;
  private mediaRecorder: any;

  constructor() {
    this.deepgram = createClient("6de4cf30c6d3833653106fe2d937c1cbf0c3d8a7");
  }

  async streamAudio(stream: MediaStream) {
    await info("streamAudio called");

    this.dgConnection = this.deepgram.listen.live({
      model: "nova",
      diarize: true,
    });

    this.mediaRecorder = new MediaRecorder(stream)

    this.mediaRecorder.addEventListener('dataavailable', async (event: { data: any; }) => {
      this.dgConnection.send(event.data)
    })

    this.dgConnection.on(LiveTranscriptionEvents.Open, () => {
      info("Connection opened.");

      this.dgConnection.on(LiveTranscriptionEvents.Close, () => {
        info("Connection closed.");
      });

      this.dgConnection.on(LiveTranscriptionEvents.Transcript, (data: { channel: { alternatives: { words: any; }[]; }; }) => {
        const words = data.channel.alternatives[0].words;
        if (words.length > 0) {
          emit('transcript', { words });
        } else {
          info('Transcript is empty');
        }
      });
    });

    this.mediaRecorder.start(1000)
  }

  async handleError(error: Error) {
    await info('handleError called', error);
    const errorMessage =
      "navigator.MediaDevices.getUserMedia error: " + error.message;
    console.error(errorMessage);
  }

  async startAudioCapture() {
    await info('startAudioCapture called');
    window.navigator.mediaDevices
      .getUserMedia({ audio: true })
      .then(stream => this.streamAudio(stream))
      .catch(error => this.handleError(error));
  }

  async stopAudioCapture() {
    await info('stopAudioCapture called');
    // Stop the MediaRecorder
    if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
      this.mediaRecorder.stop();
      this.mediaRecorder = null;
    }

    // Close the Deepgram connection
    if (this.dgConnection) {
      this.dgConnection.close();
      this.dgConnection = null;
    }
  }
}