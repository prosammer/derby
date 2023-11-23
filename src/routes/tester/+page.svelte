<script lang="ts">
  import AudioTranscriber from '$lib/audioTranscriber';
  let audioTranscriber = new AudioTranscriber();

  async function startAudioCapture() {
    await audioTranscriber.startAudioCapture();
  }

</script>
<button on:click={() => startAudioCapture()}>Start</button>