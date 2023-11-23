import { createClient, LiveTranscriptionEvents } from "@deepgram/sdk";
import { emit } from '@tauri-apps/api/event';

export default class AudioTranscriber {
  private deepgram: any;
  private dgConnection: any;
  private mediaRecorder: any;

  constructor() {
    this.deepgram = createClient("6de4cf30c6d3833653106fe2d937c1cbf0c3d8a7");
  }

  async streamAudio(stream: MediaStream) {
    console.log('streamAudio called', stream);

    this.dgConnection = this.deepgram.listen.live({ model: "nova" });

    this.mediaRecorder = new MediaRecorder(stream)

    this.mediaRecorder.addEventListener('dataavailable', async (event) => {
      this.dgConnection.send(event.data)
    })

    this.dgConnection.on(LiveTranscriptionEvents.Open, () => {
      this.dgConnection.on(LiveTranscriptionEvents.Close, () => {
        console.log("Connection closed.");
      });

      this.dgConnection.on(LiveTranscriptionEvents.Transcript, (data) => {
        const transcript = data.channel.alternatives[0].transcript;
        const speaker = data.channel.alternatives[0].words.speaker;

        if (transcript) {
          console.log("[Speaker:" + speaker + "]" + transcript);
          emit('transcript', { speaker, transcript });
        } else {
          console.log('Transcript is empty');
        }
      });
    });

    this.mediaRecorder.start(1000)
  }

  async handleError(error: Error) {
    console.log('handleError called', error);
    const errorMessage =
      "navigator.MediaDevices.getUserMedia error: " + error.message;
    console.error(errorMessage);
  }

  async startAudioCapture() {
    console.log('startAudioCapture called');
    window.navigator.mediaDevices
      .getUserMedia({ audio: true })
      .then(stream => this.streamAudio(stream))
      .catch(error => this.handleError(error));
  }
}