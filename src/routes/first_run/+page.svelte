<script lang="ts">
  import { onMount } from 'svelte';
  import { appDataDir, resolve } from "@tauri-apps/api/path";
  import { download } from "tauri-plugin-upload-api";
  import { appWindow } from "@tauri-apps/api/window";
  import { exists } from "@tauri-apps/api/fs";
  import { isPermissionGranted } from '@tauri-apps/api/notification';
  import { info, error, attachConsole } from "tauri-plugin-log-api";
  import { invoke } from "@tauri-apps/api";

  const detach = async () => {
    await attachConsole();
  }
  let downloading = false;
  let downloadSuccess = false;
  let notificationGranted = false;
  let screenRecordGranted = false;
  let audioRecordGranted = false;


export async function downloadModelFile(url: string, filename: string) {
  const appDataDirPath = await appDataDir();
  const path = await resolve(appDataDirPath, filename);

  const fileExists = await exists(path);

  if (!fileExists) {
    await info('Downloading file from ' + url + ' to ' + path);
    await download(url, path);
    await info('Download complete');
    return true;
  } else {
    await info('File already exists, skipping download');
    return true;
  }
}

async function checkNotificationPermission(): Promise<boolean> {
  let granted = await isPermissionGranted();
  if (!granted) {
    await info('No notification permissions yet, requesting permissions now');
    let result = await Notification.requestPermission();
    if (result === 'granted') {
      await info('Notification permissions granted');
      return true;
    } else {
      await error('Notification permissions denied');
      return false;
    }
  } else {
    await info('Notification permissions already granted');
    return true;
  }
}

async function checkScreenRecordingPermission(): Promise<boolean> {
  let granted = await invoke('request_screen_recording_permissions');
  if (granted) {
    await info('Screen recording permissions granted');
    return true;
  } else {
    await error('Screen recording permissions denied');
    return false;
  }
}

async function checkAudioRecordingPermission(): Promise<boolean> {
  let granted = await invoke('request_mic_permissions');
  if (granted) {
    await info('Audio recording permissions granted');
    return true;
  } else {
    await error('Audio recording permissions denied');
    return false;
  }
}

async function initialize() {
  // Start the asynchronous download task without waiting for it to finish
  downloading = true;
  const downloadTask = downloadModelFile(
    'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true',
    'ggml-base.en.bin'
  ).then(success => {
    downloading = false;
    downloadSuccess = success;
    if (success) {
      console.info('GGML Download successful');
    }
  }).catch(downloadError => {
    error('GGML Download failed: ' + downloadError);
    downloading = false;
  });


  try {
    await delay(2000);
    // Sequential permission checks with delay
    notificationGranted = await checkNotificationPermission();
    await delay(2000);
    screenRecordGranted = await checkScreenRecordingPermission();
    await delay(2000);
    audioRecordGranted = await checkAudioRecordingPermission();
  } catch (e) {
    await error('Initialization failed: ' + e);
  }

  await downloadTask;

  // Check that all permissions are granted and the download is complete
  if (notificationGranted && screenRecordGranted && audioRecordGranted && downloadSuccess) {
    await info("Successfully checked for permissions and downloaded file, closing the window.");
    await delay(2000);
    await appWindow.close();
  }
}

onMount(async () => {
  await initialize();
});

function delay(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
</script>

<div class="flex flex-col items-center justify-center h-screen p-4 bg-gray-100">
  <h1 class="text-4xl font-bold text-center mb-6">Welcome to Derby!</h1>
  <h3>Checking for permissions...</h3>
  <ul>
    <li>
      {#if notificationGranted}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
          <path fill-rule="evenodd" d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z" clip-rule="evenodd" />
        </svg>

      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {/if}
        <span>Notification permissions</span>
    </li>
    <li>
      {#if screenRecordGranted}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
          <path fill-rule="evenodd" d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z" clip-rule="evenodd" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {/if}
      <span>Screen Recording permissions</span>
    </li>
    <li>
      {#if audioRecordGranted}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" class="w-6 h-6">
          <path fill-rule="evenodd" d="M2.25 12c0-5.385 4.365-9.75 9.75-9.75s9.75 4.365 9.75 9.75-4.365 9.75-9.75 9.75S2.25 17.385 2.25 12zm13.36-1.814a.75.75 0 10-1.22-.872l-3.236 4.53L9.53 12.22a.75.75 0 00-1.06 1.06l2.25 2.25a.75.75 0 001.14-.094l3.75-5.25z" clip-rule="evenodd" />
        </svg>
      {:else}
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
          <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
      {/if}
      <span>Audio Recording permissions</span>
    </li>
  </ul>
</div>
<div class="h-10"></div>
<div class="flex flex-col items-center justify-center h-screen p-4 bg-gray-100">
{#if downloading}
  <p class="text-lg text-center mb-2">Downloading your AI model...</p>
<!--      <div class="progress-bar">-->
<!--        <div class="progress" style="width: {progressBarWidth};"></div>-->
<!--      </div>-->
{:else if downloadSuccess}
  <div class="flex items-center justify-center space-x-2">
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
      <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <span>Download successful!</span>
  </div>
{/if}
</div>
