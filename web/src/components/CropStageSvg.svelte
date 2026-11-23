<script lang="ts">
  export let stage: string = 'bibit';
  export let stressed: boolean = false;

  const STAGES = ['bibit', 'anakan', 'bunting', 'heading', 'pengisian', 'masak'];
  $: idx = Math.max(0, STAGES.indexOf(stage));
  $: stemH = 26 + idx * 14;
  $: leafColor = stressed ? '#E8734B' : idx >= 4 ? '#F3C64B' : '#7AB88A';
</script>

<svg viewBox="0 0 160 130" width="100%" height="126" role="img" aria-label={`Fase pertumbuhan ${stage}`}>
  <!-- Horizon Tanah Dasar -->
  <ellipse cx="80" cy="112" rx="46" ry="9" fill="#7A4624" opacity="0.45" />
  <ellipse cx="80" cy="110" rx="34" ry="5" fill="#9C5B34" opacity="0.7" />

  <!-- Batang Utama -->
  <line x1="80" y1="110" x2="80" y2={110 - stemH} stroke={leafColor} stroke-width="2.6" stroke-linecap="round" />

  <!-- Anakan kiri & kanan muncul mulai fase anakan (idx >= 1) -->
  {#if idx >= 1}
    <path d={`M80 108 Q64 ${102 - stemH * 0.45} 52 ${106 - stemH * 0.65}`} fill="none" stroke={leafColor} stroke-width="2" />
    <path d={`M80 108 Q96 ${102 - stemH * 0.45} 108 ${106 - stemH * 0.65}`} fill="none" stroke={leafColor} stroke-width="2" />
  {/if}

  <!-- Daun memanjang di fase bunting & heading -->
  {#if idx >= 2}
    <path d={`M80 ${106 - stemH * 0.45} Q55 ${92 - stemH * 0.6} 42 ${96 - stemH * 0.5}`} fill="none" stroke={leafColor} stroke-width="2.2" />
    <path d={`M80 ${106 - stemH * 0.5} Q105 ${92 - stemH * 0.65} 118 ${96 - stemH * 0.52}`} fill="none" stroke={leafColor} stroke-width="2.2" />
  {/if}

  <!-- Malai keluar di fase heading / pengisian / masak (idx >= 3) -->
  {#if idx >= 3}
    <path d={`M80 ${110 - stemH} Q86 ${92 - stemH} 98 ${98 - stemH * 0.88}`} fill="none" stroke={idx >= 4 ? '#F3C64B' : '#7AB88A'} stroke-width="2.2" />
    <circle cx="94" cy={96 - stemH * 0.9} r="2.2" fill={idx >= 4 ? '#F3C64B' : '#7AB88A'} />
    <circle cx="99" cy={100 - stemH * 0.87} r="2.2" fill={idx >= 4 ? '#F3C64B' : '#7AB88A'} />
  {/if}

  <text x="8" y="16" font-size="10" font-family="IBM Plex Mono, monospace" fill="currentColor">
    FASE: {stage.toUpperCase()} {stressed ? '· STRES AIR' : ''}
  </text>
</svg>
