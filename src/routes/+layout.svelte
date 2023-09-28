<script lang="ts">
	import '../app.postcss';
  import { onMount } from "svelte";
  import { appWindow } from "@tauri-apps/api/window";


  let darkMode = false;
  onMount(async() => {
    darkMode = await appWindow.theme() === "dark";
  })

</script>
<div id="darkModeDiv" class:dark={darkMode}>
  <slot/>
</div>