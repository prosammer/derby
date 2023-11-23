<script lang="ts">
  export let bubble;
</script>

{#if bubble.bot === true}
  <div class="grid grid-cols-[auto_1fr] gap-2">
    <div class="card p-4 bg-blue-200 text-black rounded-tl-none space-y-2">
      <header class="flex justify-between items-center">
        <p class="font-bold">{bubble.name}</p>
      </header>
      <p>{bubble.message}</p>
    </div>
  </div>
{:else}
  <div class="grid grid-cols-[1fr_auto] gap-2">
    <div class="card p-4 bg-green-200 text-black rounded-tr-none space-y-2 self-end">
      <header class="flex justify-between items-center">
        <p class="font-bold">{bubble.name}</p>
      </header>
      <p>{bubble.message}</p>
    </div>
  </div>
{/if}