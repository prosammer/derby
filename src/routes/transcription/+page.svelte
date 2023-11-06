<script lang="ts">
  import { invoke } from '@tauri-apps/api/tauri';
  import { onMount } from "svelte";


  // Function to emit stop_signal event
  function stopRecording() {
    invoke('set_state', { newState: 'idle' })
  }

  onMount(async () => {
    await invoke('start_recording').then((response) => {
      console.log("Start_recording finally finished:", response);
    });
  });
</script>

<div class="flex items-center justify-between bg-gray-300 p-2 rounded-lg">
  <button class="flex items-center justify-center h-10 w-10 bg-gray-400 rounded-full text-white">
    <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
    </svg>
  </button>
  <button id="stop-recording" class="flex items-center justify-center h-10 w-10 bg-red-500 rounded-full" on:click={stopRecording}></button>
</div>
