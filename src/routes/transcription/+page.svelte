<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let gptContent = 'render_spectrogram Function\n' +
    '\n' +
    'This function generates a visual representation of the spectrogram. It uses the output of the spectrogram function and renders it into a buffer that represents an image, which is then returned as a vector of bytes (Vec<u8>).\n' +
    '\n' +
    'Here\'s what happens in the render_spectrogram function:\n' +
    '\n' +
    '    Spectrogram Calculation: It first calculates the spectrogram using the previously described function.\n' +
    '    Pixel Mapping: It maps the spectral data to the pixel grid of the image to be rendered. It calculates how many pixels will represent each spectrum slice based on the provided width of the image.\n' +
    '    Frequency Mapping: It maps the frequencies to a mel scale, which is a perceptual scale of pitches, using a logarithmic transformation. This transformation involves the constants for mel scale conversion (2595 and 700).\n' +
    '    Color Mapping: It maps the magnitude of each frequency to a color using a colormap, which is an array of RGB colors. The magnitude is first converted to a decibel scale, then normalized and clamped within the range of the colormap.\n' +
    '    Image Buffer Construction: It fills an image buffer with the corresponding colors for each pixel, representing the intensity of each frequency at each point in time.';

  onMount(() => {
    let unlisten: () => void;

    // Immediately invoke the async function to listen to the event
    (async () => {
      // Listen to the global `gpt-response` event
      unlisten = await listen('gpt-response', (event) => {
        if (typeof event.payload === 'string') {
          console.log(event.payload);
          gptContent = event.payload;
        } else {
          console.error('Received non-string payload', event.payload);
        }
      });
    })();

    return () => {
      // This will remove the event listener when the component is unmounted
      if (unlisten) unlisten();
    };
  });
</script>

<div class="bg-[#07323A]/50 p-2">
  <div data-tauri-drag-region class="flex justify-between">
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="gray" class="w-5 h-5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M9.75 9.75l4.5 4.5m0-4.5l-4.5 4.5M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="gray" class="w-5 h-5">
      <path stroke-linecap="round" stroke-linejoin="round" d="M15.666 3.888A2.25 2.25 0 0013.5 2.25h-3c-1.03 0-1.9.693-2.166 1.638m7.332 0c.055.194.084.4.084.612v0a.75.75 0 01-.75.75H9a.75.75 0 01-.75-.75v0c0-.212.03-.418.084-.612m7.332 0c.646.049 1.288.11 1.927.184 1.1.128 1.907 1.077 1.907 2.185V19.5a2.25 2.25 0 01-2.25 2.25H6.75A2.25 2.25 0 014.5 19.5V6.257c0-1.108.806-2.057 1.907-2.185a48.208 48.208 0 011.927-.184" />
    </svg>
  </div>
  <div class="col-span-full text-center text-sm text-[#FDF7E3] h-64 overflow-auto">
    {gptContent}
  </div>
</div>

