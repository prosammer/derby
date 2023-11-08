<script lang="ts">
  import { onMount } from 'svelte';

  import { appDataDir, resolve } from "@tauri-apps/api/path";
  import { download } from "tauri-plugin-upload-api";
  import { writable } from "svelte/store";

  let count = 5;
  let downloading = false;
  let downloadSuccess = false;
  let intervalId: number;
  let downloadProgress = writable(0);

  export async function downloadModelFile(url: string, filename: string) {
    const appDataDirPath = await appDataDir();
    const path = await resolve(appDataDirPath, filename);

    await download(url, path, (progress, total) => {
      downloadProgress.set((progress / total) * 100); // Update the progress percentage
    });
  }


  onMount(() => {
    downloading = true;
    downloadModelFile(
      'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true',
      'ggml-base.en.bin'
    ).then(() => {
      downloading = false;
      downloadSuccess = true;
    }).catch((error: Error) => {
      console.error('Download failed:', error);
      downloading = false;
    });

    intervalId = setInterval(() => {
      if (count > 0) {
        count -= 1;
      } else {
        clearInterval(intervalId);
      }
    }, 1000);
  });
</script>

<div class="flex flex-col items-center justify-center h-screen p-4 bg-gray-100">
  <h1 class="text-4xl font-bold text-center mb-6">Welcome to Derby!</h1>
  <p class="text-lg text-center mb-2">
    {#if count > 0}
      We will ask for screen recording permissions in: {count}
    {:else if downloading}
      Downloading your AI model...
      <div class="progress-bar">
        <div class="progress" style="width: {downloadProgress}%"></div>
      </div>
    {:else if downloadSuccess}
      <div class="flex items-center justify-center space-x-2">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span>Download successful!</span>
      </div>
    {/if}
  </p>
</div>
<style>
    .progress-bar {
        width: 100%;
        background-color: #eee;
    }
    .progress {
        height: 20px;
        background-color: #4CAF50;
        width: 0;
        transition: width 0.5s;
    }
</style>
