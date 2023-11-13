<div class="typing-animation text-[#FDF7E3]">
  <div class="dot">.</div>
  <div class="dot">.</div>
  <div class="dot">.</div>
</div>
<style>
    .typing-animation {
        display: flex;
        align-items: center;
    }

    .dot {
        border-radius: 50%;
        height: 3px;
        margin: 0 2px;
        animation: dot-wave 1.5s infinite;
    }

    .dot:nth-child(1) { animation-delay: 0s; }
    .dot:nth-child(2) { animation-delay: 0.15s; }
    .dot:nth-child(3) { animation-delay: 0.3s; }

    /* Keyframes for the wave animation */
    @keyframes dot-wave {
        0%, 60%, 100% {
            transform: translateY(0);
            transform: scale(1);
        }
        30% {
            transform: translateY(-15px);
            transform: scale(1.2);
        }
    }
</style>