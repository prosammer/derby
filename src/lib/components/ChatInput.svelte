<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  const dispatch = createEventDispatcher();

  export let currentMessage = '';

  function onPromptKeydown(event: KeyboardEvent): void {
    if (['Enter'].includes(event.code)) {
      event.preventDefault();
      dispatch('send', currentMessage);
      currentMessage = '';
    }
  }
</script>

<section class="border-t border-surface-500/30 p-4">
  <div class="input-group input-group-divider grid-cols-[auto_1fr_auto] rounded-container-token">
    <button class="input-group-shim">+</button>
    <textarea
      bind:value={currentMessage}
      class="bg-transparent border-0 ring-0"
      name="prompt"
      id="prompt"
      placeholder="Write a message..."
      rows="1"
      on:keydown={onPromptKeydown}
    />
    <button class={currentMessage ? 'variant-filled-primary' : 'input-group-shim'}>
      <i class="fa-solid fa-paper-plane" />
    </button>
  </div>
</section>