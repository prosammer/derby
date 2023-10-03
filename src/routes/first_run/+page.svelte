<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/tauri';

  let count = 5;
  let intervalId: number;

  onMount(() => {
    intervalId = setInterval(() => {
      if (count > 0) {
        count -= 1;
      } else {
        clearInterval(intervalId);
        invoke('request_screen_recording_permissions').then((result) => console.log(result))
      }
    }, 1000);
  });
</script>

<h1>Welcome to octoguide!</h1>
<h2>We will ask for screen recording permissions in: {count}</h2>
