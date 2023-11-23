<script lang="ts">
  import type { Message } from "$lib/types/message";
  export let bubble: Message;
</script>

{#if bubble.speaker === 0}
  <div class="grid grid-cols-[auto_1fr] gap-2">
    <div class="card p-4 bg-blue-200 text-black rounded-tl-none space-y-2">
      <header class="flex justify-between items-center">
        <p class="font-bold">{bubble.speaker}</p>
      </header>
      <p>{bubble.content}</p>
    </div>
  </div>
{:else}
  <div class="grid grid-cols-[1fr_auto] gap-2">
    <div class="card p-4 bg-green-200 text-black rounded-tr-none space-y-2 self-end">
      <header class="flex justify-between items-center">
        <p class="font-bold">{bubble.speaker}</p>
      </header>
      <p>{bubble.content}</p>
    </div>
  </div>
{/if}