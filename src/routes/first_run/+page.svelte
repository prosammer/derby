<script lang="ts">
  import { onMount } from 'svelte';
  import { appDataDir, resolve } from "@tauri-apps/api/path";
  import { download } from "tauri-plugin-upload-api";
  import { appWindow } from "@tauri-apps/api/window";
  import { exists } from "@tauri-apps/api/fs";
  import { isPermissionGranted } from '@tauri-apps/api/notification';
  import { info, error, attachConsole } from "tauri-plugin-log-api";
  import { invoke } from "@tauri-apps/api";
  import { CheckCircle2, CircleDashed} from 'lucide-svelte';

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

<div class="flex flex-col items-center justify-center h-screen p-4 bg-gray-100 space-y-6">
  <h1 class="text-4xl font-bold text-center">Welcome to Derby!</h1>
  <h3>Checking for permissions...</h3>
  <ul class="space-y-2">
    <li class="flex items-center">
      {#if notificationGranted}
        <CheckCircle2/>
      {:else}
        <CircleDashed />
      {/if}
      <span class="ml-2">Notification permissions</span>
    </li>
    <li class="flex items-center">
      {#if screenRecordGranted}
        <CheckCircle2/>
      {:else}
        <CircleDashed />
      {/if}
      <span class="ml-2">Screen Recording permissions</span>
    </li>
    <li class="flex items-center">
      {#if audioRecordGranted}
        <CheckCircle2/>
      {:else}
        <CircleDashed />
      {/if}
      <span class="ml-2">Audio Recording permissions</span>
    </li>
    <li class="flex items-center">
      {#if downloadSuccess}
        <CheckCircle2 />
      {:else}
        <CircleDashed id="spinning_downloading"  class="animate-spin duration-3000 ease-linear"/>
      {/if}
      <span class="ml-2">AI model downloaded</span>
    </li>
  </ul>
</div>