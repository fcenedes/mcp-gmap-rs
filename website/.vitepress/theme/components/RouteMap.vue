<script setup lang="ts">
const roads = Array.from({ length: 27 }, (_, i) => ({
  d: `M ${170 + i * 39} -60 Q ${290 + i * 24} 275 ${90 + i * 39} 850`,
  width: i % 5 === 0 ? 5 : 1.3,
}));
const cross = Array.from({ length: 21 }, (_, i) => ({
  d: `M 20 ${-70 + i * 45} Q 650 ${120 + i * 30} 1270 ${-80 + i * 45}`,
  width: i % 6 === 0 ? 5 : 1.3,
}));
</script>

<template>
  <svg class="route-map" viewBox="0 0 1280 820" aria-hidden="true">
    <defs>
      <radialGradient id="map-light">
        <stop stop-color="#24342b" stop-opacity=".55" />
        <stop offset="1" stop-color="#151a18" stop-opacity="0" />
      </radialGradient>
      <filter id="route-glow"><feGaussianBlur stdDeviation="9" /></filter>
      <pattern
        id="map-dots"
        width="27"
        height="27"
        patternUnits="userSpaceOnUse"
      >
        <circle cx="1" cy="1" r=".65" fill="#859188" opacity=".28" />
      </pattern>
    </defs>
    <rect width="1280" height="820" fill="url(#map-light)" />
    <rect width="1280" height="820" fill="url(#map-dots)" />
    <g fill="#25342b" opacity=".7">
      <path d="m460 150 85 11-14 118-68-12Z" />
      <path d="m900 360 77-6 22 121-81 11Z" />
      <path d="m1040 610 128 34-43 120-113-40Z" />
    </g>
    <g fill="none" stroke="#344039">
      <path
        v-for="(road, i) in roads"
        :key="`r${i}`"
        :d="road.d"
        :stroke-width="road.width"
      />
      <path
        v-for="(road, i) in cross"
        :key="`c${i}`"
        :d="road.d"
        :stroke-width="road.width"
      />
    </g>
    <path
      d="M 70 670 C 270 590 270 370 480 395 S 650 680 820 586 930 330 1290 410"
      fill="none"
      stroke="#182421"
      stroke-width="51"
    />
    <path
      d="M 70 670 C 270 590 270 370 480 395 S 650 680 820 586 930 330 1290 410"
      fill="none"
      stroke="#4a6155"
      stroke-width=".7"
    />
    <g fill="none" stroke="#7f8c7f" stroke-width="7">
      <path d="m442 359 9 80" />
      <path d="m681 581 54-28" />
      <path d="m864 495 51 19" />
    </g>
    <g fill="none" stroke="#68766a" stroke-width=".8">
      <circle cx="707" cy="282" r="180" stroke-dasharray="3 9" />
      <circle cx="707" cy="282" r="270" opacity=".45" stroke-dasharray="3 9" />
    </g>
    <path
      d="M 450 428 462 316 577 340 599 250 707 282 743 156 889 192 911 289 993 312"
      fill="none"
      stroke="#f3b664"
      stroke-width="10"
      opacity=".15"
      filter="url(#route-glow)"
    />
    <path
      class="map-route"
      d="M 450 428 462 316 577 340 599 250 707 282 743 156 889 192 911 289 993 312"
      fill="none"
      stroke="#f3b664"
      stroke-width="3"
      stroke-linejoin="round"
    />
    <g fill="#f3b664">
      <circle cx="450" cy="428" r="7" />
      <circle cx="707" cy="282" r="5" />
      <circle cx="993" cy="312" r="7" />
    </g>
    <g stroke="#f3b664" fill="#151a18">
      <circle cx="450" cy="428" r="12" />
      <circle cx="450" cy="428" r="4" fill="#f3b664" />
      <circle cx="993" cy="312" r="12" />
      <circle cx="993" cy="312" r="4" fill="#f3b664" />
    </g>
    <g
      font-family="'IBM Plex Mono',monospace"
      font-size="11"
      letter-spacing="2"
    >
      <text x="420" y="470" fill="#f3b664">48.8584, 2.2945</text>
      <text x="953" y="351" fill="#f3b664">48.8606, 2.3376</text>
      <text x="730" y="323" fill="#a8b1a5">PARIS</text>
      <text x="639" y="655" fill="#63746a" transform="rotate(24 639 655)">
        LA SEINE
      </text>
      <text x="1020" y="150" fill="#7e8b80">N ↑</text>
    </g>
    <g
      transform="translate(1060 620)"
      stroke="#7e8b80"
      fill="none"
      stroke-width=".8"
    >
      <circle r="35" />
      <path d="M0-43V43M-43 0H43M0-28 8 0 0-8-8 0Z" />
      <text
        x="-3"
        y="-51"
        fill="#9da89d"
        stroke="none"
        font-family="monospace"
        font-size="10"
      >
        N
      </text>
    </g>
  </svg>
</template>
