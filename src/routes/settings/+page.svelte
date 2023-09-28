<script lang="ts">
  import { onMount } from 'svelte';
  import { Label } from "$components/ui/label";
  import { Checkbox } from "$components/ui/checkbox";
  import Textarea from "$components/ui/textarea/Textarea.svelte"

  import { Store } from "tauri-plugin-store-api";
  import { enable, disable } from "tauri-plugin-autostart-api";

  const store = new Store(".settings.dat");

  let time: string;
  let startOnLogin: boolean;
  let userPrompt: string;
  let userFirstName: string;


  onMount(async () => {
    time= await store.get("time") || "15:00";
    startOnLogin= await store.get("startOnLogin") || false;
    userPrompt = await store.get("userPrompt") || "1.Shower\n2.Brush Teeth\n3.Make Bed";
    userFirstName= await store.get("userFirstName") || "User";
  });

  $: store.set("time", time).then(() => store.save())
  $: startOnLogin ? enable() : disable();
  $: store.set("startOnLogin", startOnLogin).then(() => store.save())
  $: store.set("userPrompt", userPrompt).then(() => store.save())
  $: store.set("userFirstName", userFirstName).then(() => store.save())

</script>
<div class="w-full h-full dark:bg-[#2C2831]">
  <div class="w-5/6 mx-auto p-5 shadow-lg">
    <h1 class="pb-4 dark:text-white">General</h1>
    <div class="mb-4">
    </div>

    <div class="mb-4 flex items-center">
      <Checkbox bind:checked={startOnLogin} id="startOnLogin" class="dark:outline-dark-mode-white" />
      <Label for="startOnLogin" class="ml-2 dark:text-white">Start on Login</Label>
    </div>
    <div class="mb-4 flex items-center">
      <Label for="userPrompt" class="px-2 dark:text-white">User Prompt</Label>
      <Textarea bind:value={userPrompt} placeholder="1.Shower&#10;2.Brush Teeth&#10;3.Make Bed" class="dark:text-white dark:border-dark-mode-white"></Textarea>
    </div>
    <div class="mb-4 flex items-center">
      <Label for="userFirstName" class="px-2 dark:text-white">First Name</Label>
      <p>This is just given to the bot so that it can communicate with you clearly</p>
      <input type="text" bind:value={userFirstName} placeholder="John" class="dark:border-dark-mode-white" />
    </div>
    <div class="h-96">
    </div>
  </div>
</div>