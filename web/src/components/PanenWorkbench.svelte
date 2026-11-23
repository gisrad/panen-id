<script lang="ts">
  import { onMount, onDestroy, tick } from 'svelte';
  import maplibregl from 'maplibre-gl';
  import uPlot from 'uplot';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import 'uplot/dist/uPlot.min.css';
  import CropStageSvg from './CropStageSvg.svelte';
  import TrialOverlay from './TrialOverlay.svelte';
  import SoilBlock3D from './SoilBlock3D.svelte';
  import LahanSayaPanel from './LahanSayaPanel.svelte';
  import { computeGddSeries, computeOlsSummary, type OlsSummary } from '../lib/agroWasm';

  interface SoilHorizon {
    depth_cm: string;
    clay_pct: number;
    sand_pct: number;
    soc_g_kg: number;
    ph_h2o: number;
    cec_cmol_kg: number;
  }

  interface PlotProperties {
    plot_id: string;
    zone: string;
    yield_t_ha: number;
    clay_pct: number;
    ndvi_peak: number;
    drainage: string;
    shallow_note: string;
    deep_note: string;
  }

  interface FieldBundle {
    field_id: string;
    name: string;
    regency: string;
    linked_kode_bps: string;
    season_id: string;
    crop: string;
    variety: string;
    is_simulated: boolean;
    center: [number, number];
    dates: string[];
    rain_mm: number[];
    tmax_c: number[];
    gdd_cum: number[];
    stage_code: string[];
    ndvi_s2_hls: number[];
    stress_windows: { start: string; end: string; kind: string }[];
    soil_profile: SoilHorizon[];
    plots: GeoJSON.FeatureCollection<GeoJSON.Polygon, PlotProperties>;
  }

  let fieldId: 'bompon-magelang' | 'klaten-irigasi' = 'bompon-magelang';
  let bundle: FieldBundle | null = null;
  let activeTab: 'replay' | 'analisis' | 'tanah3d' | 'lahan' = 'replay';
  let mapLayer: 'yield' | 'clay' | 'ndvi' = 'yield';
  let isEmbed = false;
  let copyNotice = '';
  let wasmGddFinal: number | null = null;
  let fieldOls: OlsSummary | null = null;
  let dayIdx = 74;
  let playing = false;
  let timer: number | null = null;
  let selectedPlotId = 'P1-ATAS';
  let theme: 'malam' | 'kertas' = 'kertas';

  let remainingSec = 300;
  let authenticated = false;
  let userEmail = '';
  let wallActive = false;

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let chartContainer: HTMLDivElement;
  let chart: uPlot | null = null;

  $: activeDate = bundle?.dates[dayIdx] || '2025-07-15';
  $: activeStage = bundle?.stage_code[dayIdx] || 'heading';
  $: activeStressed = Boolean(
    bundle?.stress_windows.some((w) => activeDate >= w.start && activeDate <= w.end)
  );
  $: selectedPlot =
    bundle?.plots.features.find((f) => f.properties.plot_id === selectedPlotId)?.properties ||
    bundle?.plots.features[0]?.properties ||
    null;

  function formatMmSs(sec: number): string {
    const s = Math.max(0, Math.floor(sec));
    return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
  }

  function plotColor(p: PlotProperties): string {
    if (mapLayer === 'yield') {
      // Crameri batlow inspirasi
      if (p.yield_t_ha >= 6.8) return '#011959';
      if (p.yield_t_ha >= 6.0) return '#4A6E5C';
      if (p.yield_t_ha >= 5.2) return '#8F8B45';
      return '#D6995C';
    }
    if (mapLayer === 'clay') {
      // Crameri lajolla
      if (p.clay_pct >= 48) return '#69271B';
      if (p.clay_pct >= 35) return '#BD5319';
      return '#E69135';
    }
    // Crameri bamako (NDVI)
    if (p.ndvi_peak >= 0.78) return '#003B47';
    if (p.ndvi_peak >= 0.70) return '#265F38';
    return '#B3A134';
  }

  function updateMapPlots() {
    if (!map || !bundle) return;
    const enriched: GeoJSON.FeatureCollection = {
      type: 'FeatureCollection',
      features: bundle.plots.features.map((f) => ({
        ...f,
        properties: {
          ...f.properties,
          fillColor: plotColor(f.properties),
          isSelected: f.properties.plot_id === selectedPlotId ? 1 : 0,
        },
      })),
    };
    const src = map.getSource('panen-plots') as maplibregl.GeoJSONSource | undefined;
    if (src) src.setData(enriched);
  }

  function renderChart() {
    if (!chartContainer || !bundle) return;
    if (chart) {
      chart.destroy();
      chart = null;
    }
    const xs = bundle.dates.map((_d, i) => i);
    const gddScaled = bundle.gdd_cum.map((g) => Number((g / 40).toFixed(1)));
    chart = new uPlot(
      {
        width: Math.max(300, chartContainer.clientWidth || 440),
        height: 155,
        cursor: { drag: { x: false, y: false } },
        scales: { x: { time: false }, y: { range: [0, 50] } },
        axes: [
          {
            stroke: '#6B6B73',
            font: '10px IBM Plex Mono, monospace',
            values: (_u, vals) => vals.map((v) => bundle?.dates[Math.round(v)]?.slice(5) || ''),
          },
          { stroke: '#6B6B73', font: '10px IBM Plex Mono, monospace', size: 32 },
        ],
        series: [
          {},
          { label: 'Hujan (mm)', stroke: '#4E7A96', width: 1.5 },
          { label: 'Tmax (°C)', stroke: '#E8734B', width: 1.6 },
          { label: 'GDD/40', stroke: '#12703B', width: 1.8 },
        ],
      },
      [xs, bundle.rain_mm, bundle.tmax_c, gddScaled],
      chartContainer
    );
  }

  async function loadField(id: 'bompon-magelang' | 'klaten-irigasi') {
    fieldId = id;
    const file = id === 'bompon-magelang' ? 'bompon-mt2-2025.json' : 'klaten-mt2-2025.json';
    const res = await fetch(`/ulang/mock-data/${file}`);
    bundle = await res.json();
    selectedPlotId = bundle?.plots.features[0]?.properties.plot_id || '';
    dayIdx = Math.min(74, (bundle?.dates.length || 90) - 1);
    await tick();
    if (map && bundle) {
      map.flyTo({ center: bundle.center, zoom: 15.2, duration: 700 });
      updateMapPlots();
    }
    renderChart();
    if (bundle) {
      computeGddSeries(bundle.tmax_c, bundle.tmin_c)
        .then((g) => {
          if (g.length > 0) wasmGddFinal = g[g.length - 1];
        })
        .catch(() => {});
      const clays = bundle.plots.features.map((f) => f.properties.clay_pct);
      const ylds = bundle.plots.features.map((f) => f.properties.yield_t_ha);
      computeOlsSummary(clays, ylds)
        .then((res) => {
          fieldOls = res;
        })
        .catch(() => {});
    }
  }

  $: if (map && bundle) {
    mapLayer;
    selectedPlotId;
    updateMapPlots();
  }

  function togglePlay() {
    if (playing) {
      if (timer) window.clearInterval(timer);
      timer = null;
      playing = false;
      return;
    }
    playing = true;
    timer = window.setInterval(() => {
      if (!bundle) return;
      dayIdx = dayIdx >= bundle.dates.length - 1 ? 0 : dayIdx + 1;
    }, 240);
  }

  function toggleTheme() {
    theme = theme === 'kertas' ? 'malam' : 'kertas';
    document.documentElement.setAttribute('data-theme', theme);
    window.localStorage.setItem('agri_theme', theme);
  }

  function buildPermalink(): string {
    const u = new URL(window.location.href);
    u.searchParams.set('tab', activeTab);
    u.searchParams.set('layer', mapLayer);
    u.searchParams.set('day', String(dayIdx));
    if (selectedPlotId) u.searchParams.set('plot', selectedPlotId);
    u.searchParams.delete('embed');
    return u.toString();
  }

  async function copyPermalink() {
    const urlStr = buildPermalink();
    window.history.replaceState(null, '', urlStr);
    try {
      await navigator.clipboard.writeText(urlStr);
      copyNotice = 'Permalink tersalin ke papan klip.';
    } catch {
      copyNotice = urlStr;
    }
  }

  async function copyEmbed() {
    const u = new URL(buildPermalink());
    u.searchParams.set('embed', '1');
    const code = `<iframe src="${u.toString()}" width="100%" height="680" style="border:1px solid #E7E7E7" loading="lazy" title="Panen-ID Musim Ulang"></iframe>`;
    try {
      await navigator.clipboard.writeText(code);
      copyNotice = 'Kode embed <iframe> tersalin.';
    } catch {
      copyNotice = code;
    }
  }

  function exportSubsetGeoJson() {
    if (!bundle) return;
    const fc = {
      type: 'FeatureCollection',
      name: `panen_id_${bundle.field_id}_${bundle.season_id}`,
      metadata: {
        source: 'agri.transbulent.id/ulang',
        attribution: 'Agri Transbulent / Transbulent Research Group (agri.transbulent.id)',
        field_id: bundle.field_id,
        season_id: bundle.season_id,
        regency: bundle.regency,
        day_date: activeDate,
        gdd_cum: bundle.gdd_cum[dayIdx] ?? 0,
        is_simulated: bundle.is_simulated,
      },
      features: bundle.plots.features.map((f) => ({
        ...f,
        properties: {
          ...f.properties,
          field_id: bundle.field_id,
          season_id: bundle.season_id,
          day_date: activeDate,
          is_simulated: bundle.is_simulated,
        },
      })),
    };
    const blob = new Blob([JSON.stringify(fc, null, 2)], { type: 'application/geo+json;charset=utf-8' });
    const a = document.createElement('a');
    a.href = URL.createObjectURL(blob);
    a.download = `${bundle.field_id}-${bundle.season_id}-subset.geojson`;
    a.click();
    URL.revokeObjectURL(a.href);
  }

  onMount(async () => {
    const qs = new URLSearchParams(window.location.search);
    isEmbed = qs.get('embed') === '1';

    const savedTheme = window.localStorage.getItem('agri_theme');
    if (savedTheme === 'malam' || savedTheme === 'kertas') {
      theme = savedTheme;
    } else {
      theme = 'kertas';
    }
    document.documentElement.setAttribute('data-theme', theme);

    if (window.location.pathname.includes('klaten')) fieldId = 'klaten-irigasi';
    if (window.location.pathname.includes('lahan-saya')) activeTab = 'lahan';
    await loadField(fieldId);

    const qTab = qs.get('tab');
    if (qTab === 'replay' || qTab === 'analisis' || qTab === 'tanah3d' || qTab === 'lahan') activeTab = qTab;
    const qLayer = qs.get('layer');
    if (qLayer === 'yield' || qLayer === 'clay' || qLayer === 'ndvi') mapLayer = qLayer;
    const qDay = Number(qs.get('day'));
    if (Number.isInteger(qDay) && bundle && qDay >= 0 && qDay < bundle.dates.length) dayIdx = qDay;
    const qPlot = qs.get('plot');
    if (qPlot && bundle?.plots.features.some((f) => f.properties.plot_id === qPlot)) selectedPlotId = qPlot;

    map = new maplibregl.Map({
      container: mapContainer,
      style: 'https://tiles.openfreemap.org/styles/positron',
      center: bundle?.center || [110.078, -7.548],
      zoom: 15.2,
      attributionControl: false,
    });
    map.addControl(new maplibregl.NavigationControl({ showCompass: false }), 'bottom-right');

    map.on('load', () => {
      if (!map || !bundle) return;
      map.addSource('panen-plots', { type: 'geojson', data: bundle.plots });
      map.addLayer({
        id: 'panen-fill',
        type: 'fill',
        source: 'panen-plots',
        paint: {
          'fill-color': ['coalesce', ['get', 'fillColor'], '#4A6E5C'],
          'fill-opacity': 0.78,
        },
      });
      map.addLayer({
        id: 'panen-line',
        type: 'line',
        source: 'panen-plots',
        paint: {
          'line-color': ['case', ['==', ['get', 'isSelected'], 1], '#000000', '#FFFFFF'],
          'line-width': ['case', ['==', ['get', 'isSelected'], 1], 2.5, 1.0],
        },
      });
      updateMapPlots();

      map.on('click', 'panen-fill', (e) => {
        const pid = e.features?.[0]?.properties?.plot_id;
        if (pid) selectedPlotId = String(pid);
      });
    });
  });

  onDestroy(() => {
    if (timer) window.clearInterval(timer);
    if (chart) chart.destroy();
    if (map) map.remove();
  });
</script>

<div style="min-height: 100vh; display: flex; flex-direction: column;">
  {#if !isEmbed}
    <div class="agri-spectrum-banner">
      MUSIM ULANG (PANEN-ID) · HASIL PANEN × 6 HORIZON TANAH × CUACA GDD + FAO-56 · SIMULASI MOCK P1
    </div>

    <header class="topbar">
      <div class="topbar-inner">
        <a href="/" class="brand-link">
          <span class="brand-box">AG</span>
          <span>agri.transbulent.id</span>
        </a>

        <nav class="hnav" aria-label="Navigasi">
          <a href="/">Beranda</a>
          <a href="/ulang/bompon-magelang/mt2-2025/" class="on">Musim Ulang</a>
          <a href="/kekeringan/">Kekeringan</a>
          <a href="/metode/">Metode</a>
          <a href="/lobby/">Lobby</a>
        </nav>

        <div style="display: flex; align-items: center; gap: 8px; margin-left: auto; flex-wrap: wrap;">
          <button
            type="button"
            class={fieldId === 'bompon-magelang' ? 'btn-primary' : 'btn-subtle'}
            on:click={() => loadField('bompon-magelang')}
          >
            DAS Bompon
          </button>
          <button
            type="button"
            class={fieldId === 'klaten-irigasi' ? 'btn-primary' : 'btn-subtle'}
            on:click={() => loadField('klaten-irigasi')}
          >
            Irigasi Klaten
          </button>
          {#if authenticated}
            <span class="live-badge"><span class="live-dot"></span><span>{userEmail}</span></span>
          {:else}
            <a href="/lobby/" class="live-badge">
              <span class={`live-dot ${remainingSec <= 60 ? 'is-warn' : ''}`}></span>
              <span>Pratinjau</span>
              <b>{formatMmSs(remainingSec)}</b>
            </a>
          {/if}
          <button type="button" class="btn-subtle" on:click={toggleTheme}>
            {theme === 'kertas' ? 'Malam' : 'Putih'}
          </button>
        </div>
      </div>
    </header>
  {/if}

  <main style="flex: 1; display: grid; grid-template-columns: 1.25fr 0.95fr; min-height: 620px;">
    <!-- Kolom Kiri: Peta + Timeline Scrubber + uPlot -->
    <section style="display: grid; grid-template-rows: 1fr auto auto; border-right: 1px solid var(--border);">
      <div style="position: relative; min-height: 360px;">
        <div bind:this={mapContainer} style="position: absolute; inset: 0;"></div>

        <div style="position: absolute; top: 12px; left: 12px; z-index: 10; background: var(--dock-bg); border: 1.5px solid var(--border-ink); padding: 10px; display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
          <span class="pill-badge pill-sim">SIMULASI</span>
          {#each (['yield', 'clay', 'ndvi'] as const) as k}
            <button
              type="button"
              class={mapLayer === k ? 'btn-primary' : 'btn-subtle'}
              style="padding: 6px 9px; font-size: 10px;"
              on:click={() => (mapLayer = k)}
            >
              {k === 'yield' ? 'Hasil (batlow)' : k === 'clay' ? '% Liat (lajolla)' : 'NDVI (bamako)'}
            </button>
          {/each}
        </div>
      </div>

      <!-- Scrubber 120 Hari -->
      <div style="border-top: 1px solid var(--border); padding: 12px 16px; background: var(--bg); display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
        <button type="button" class="btn-primary" style="padding: 7px 12px;" on:click={togglePlay}>
          {playing ? 'Jeda' : 'Putar Musim'}
        </button>
        <input
          type="range"
          min="0"
          max={Math.max(0, (bundle?.dates.length || 120) - 1)}
          bind:value={dayIdx}
          style="flex: 1; accent-color: #000000;"
          aria-label="Scrubber hari tanam"
        />
        <span class="num" style="font-size: 12px;">
          H+{dayIdx + 1} · {activeDate} · GDD {bundle?.gdd_cum[dayIdx] ?? 0}
        </span>
      </div>

      <!-- Grafik uPlot -->
      <div style="border-top: 1px solid var(--border); padding: 10px 16px; background: var(--surface);">
        <div class="num" style="font-size: 10px; color: var(--ink-muted); margin-bottom: 4px;">
          HUJAN HARIAN (BIRU), TMAX °C (TERAKOTA), &amp; AKUMULASI GDD/40 (HIJAU)
        </div>
        <div bind:this={chartContainer}></div>
      </div>
    </section>

    <!-- Kolom Kanan: Tanaman SVG + 6 Horizon SoilGrids + Lensa Dangkal vs Dalam -->
    <aside style="padding: 18px; display: grid; gap: 16px; align-content: start; overflow-y: auto; max-height: calc(100vh - 90px);">
      <div style="display: flex; gap: 6px; flex-wrap: wrap;">
        <button type="button" class={activeTab === 'replay' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'replay')}>
          Replay &amp; Tanah
        </button>
        <button type="button" class={activeTab === 'analisis' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'analisis')}>
          Lensa Dangkal vs Dalam
        </button>
        <button type="button" class={activeTab === 'tanah3d' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'tanah3d')}>
          Blok 3D Tanah
        </button>
        <button type="button" class={activeTab === 'lahan' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'lahan')}>
          Lahan Saya (BYOD)
        </button>
      </div>

      {#if wasmGddFinal !== null || fieldOls}
        <div style="border: 1px solid var(--border); padding: 8px 10px; background: var(--surface); display: flex; justify-content: space-between; flex-wrap: wrap; gap: 6px; font-size: 11px;">
          <span class="num">WASM GDD 120H: {wasmGddFinal !== null ? wasmGddFinal.toFixed(0) : '-'}</span>
          {#if fieldOls}
            <span class="num" style={fieldOls.sampleGuardTriggered ? 'color: #B83B14;' : ''}>
              OLS Liat→Hasil n={fieldOls.n} {fieldOls.sampleGuardTriggered ? '(Guard n<30)' : `R²=${fieldOls.r2.toFixed(2)}`}
            </span>
          {/if}
        </div>
      {/if}

      {#if bundle && selectedPlot}
        <div>
          <div class="num" style="font-size: 10.5px; color: var(--ink-muted);">
            {bundle.name} · {bundle.regency} · Varietas {bundle.variety}
          </div>
          <h1 style="margin: 4px 0 0; font-size: 1.35rem; font-weight: 600;">
            Petak {selectedPlot.plot_id} — {selectedPlot.zone}
          </h1>
        </div>

        {#if activeTab === 'replay'}
          <div style="display: grid; grid-template-columns: 0.9fr 1.1fr; gap: 12px; border: 1px solid var(--border); padding: 12px; background: var(--surface);">
            <CropStageSvg stage={activeStage} stressed={activeStressed} />
            <div style="display: grid; gap: 6px; align-content: center;">
              <div class="num" style="font-size: 10.5px; color: var(--ink-muted);">HASIL PETAK (SIMULASI)</div>
              <div class="num" style="font-size: 1.6rem; font-weight: 600;">{selectedPlot.yield_t_ha} t/ha</div>
              <div class="num" style="font-size: 11px;">Liat: {selectedPlot.clay_pct}% · NDVI Puncak: {selectedPlot.ndvi_peak}</div>
              <div class="num" style="font-size: 11px; color: var(--ink-muted);">Drainase: {selectedPlot.drainage}</div>
            </div>
          </div>

          <div>
            <div class="num" style="font-size: 10.5px; color: var(--ink-muted); margin-bottom: 6px;">
              PROFIL 6 HORIZON TANAH SOILGRIDS (0–200 CM)
            </div>
            <div style="display: grid; gap: 4px;">
              {#each bundle.soil_profile as h}
                <div style="display: grid; grid-template-columns: 74px 1fr auto; gap: 10px; align-items: center; padding: 6px 8px; border: 1px solid var(--border); font-size: 11.5px;">
                  <span class="num">{h.depth_cm} cm</span>
                  <div style="height: 8px; background: var(--border); position: relative;">
                    <div style={`height: 100%; width: ${h.clay_pct}%; background: #B83B14;`}></div>
                  </div>
                  <span class="num">Liat {h.clay_pct}% · pH {h.ph_h2o}</span>
                </div>
              {/each}
            </div>
          </div>
        {:else if activeTab === 'analisis'}
          <div style="border: 1px solid var(--border); padding: 14px; display: grid; gap: 10px;">
            <div class="pill-badge">LENSA DANGKAL (HASIL ~ VARIETAS)</div>
            <p style="margin: 0; font-size: 0.92rem;">{selectedPlot.shallow_note}</p>
          </div>
          <div style="border: 1.5px solid var(--border-ink); padding: 14px; background: var(--surface); display: grid; gap: 10px;">
            <div class="pill-badge pill-warn">LENSA DALAM (VARIETAS × TANAH × GDD)</div>
            <p style="margin: 0; font-size: 0.94rem; font-weight: 600;">{selectedPlot.deep_note}</p>
          </div>
        {:else if activeTab === 'tanah3d'}
          <SoilBlock3D
            soilProfile={bundle.soil_profile}
            plots={bundle.plots.features}
            {selectedPlotId}
            fieldName={bundle.name}
          />
        {:else}
          <LahanSayaPanel />
        {/if}

        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 6px; border-top: 1px solid var(--border); padding-top: 10px;">
          <button type="button" class="btn-subtle" on:click={copyPermalink}>Salin Permalink</button>
          <button type="button" class="btn-subtle" on:click={copyEmbed}>Salin Embed</button>
          <button type="button" class="btn-subtle" on:click={exportSubsetGeoJson}>Ekspor GeoJSON</button>
        </div>
        {#if copyNotice}
          <div class="num" style="font-size: 10.5px; color: var(--ink-muted); word-break: break-all;">{copyNotice}</div>
        {/if}

        <a href={`/kekeringan/${bundle.linked_kode_bps}/`} class="btn-outline">
          Lihat Konteks Kekeringan {bundle.regency} →
        </a>
      {/if}
    </aside>
  </main>

  <TrialOverlay bind:remainingSec bind:authenticated bind:userEmail bind:wallActive />
</div>
