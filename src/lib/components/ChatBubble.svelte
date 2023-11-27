<script lang="ts">

  import type { Message } from "ai";

  export let message: Message;
</script>

{#if message.role === 'user'}
  <div class="grid grid-cols-[auto_1fr] gap-2">
    <div class="card p-4 variant-soft rounded-tl-none space-y-2 variant-soft-primary">
      <header class="flex justify-between items-center">
      </header>
      <p>{message.content}</p>
    </div>
  </div>
{:else}
  <div class="grid grid-cols-[1fr_auto] gap-2">
    <div class="card p-4 rounded-tr-none space-y-2 variant-soft-primary">
      <header class="flex justify-between items-center">
      </header>
      <p>{message.content}</p>
    </div>
  </div>
{/if}