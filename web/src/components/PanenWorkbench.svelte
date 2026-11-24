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
  import {
    computeGddSeries,
    computeKmeansZones,
    computeOlsSummary,
    computePhenologySeries,
    computeYieldExplain,
    CROP_PROFILES,
    type CropKey,
    type OlsSummary,
    type YieldExplainResult,
  } from '../lib/agroWasm';

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
    morphology?: string;
    area_ha?: number;
    yield_t_ha: number;
    clay_pct: number;
    silt_pct?: number;
    sand_pct?: number;
    permeability?: number;
    landslide_class?: string;
    yield_2024?: number;
    yield_2025?: number;
    yoy_delta_t_ha?: number;
    stability_class?: 'stabil_tinggi' | 'sensitif_iklim' | 'rentan_kronis';
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
    tmin_c: number[];
    et0_mm?: number[];
    weather_source?: string;
    gdd_cum: number[];
    stage_code: string[];
    ndvi_s2_hls: number[];
    stress_windows: { start: string; end: string; kind: string }[];
    soil_profile: SoilHorizon[];
    plots: GeoJSON.FeatureCollection<GeoJSON.Polygon, PlotProperties>;
  }

  let fieldId: 'bompon-magelang' | 'klaten-irigasi' = 'bompon-magelang';
  let seasonId: 'mt2-2025' | 'mt2-2024' = 'mt2-2025';
  let bundle: FieldBundle | null = null;
  let activeTab: 'replay' | 'analisis' | 'tanah3d' | 'lahan' = 'replay';
  let mapLayer: 'yield' | 'clay' | 'ndvi' | 'stability' | 'zones' = 'yield';
  let isEmbed = false;
  let copyNotice = '';
  let cropKey: CropKey = 'padi';
  let wasmGddSeries: number[] = [];
  let wasmStageSeries: number[] = [];
  let wasmGddFinal: number | null = null;
  let fieldOls: OlsSummary | null = null;
  let zoneMap: Record<string, number> = {};
  let zoneSep = 0;
  let yieldExplain: YieldExplainResult | null = null;
  let dayIdx = 74;
  let playing = false;
  let timer: number | null = null;
  let selectedPlotId = 'BP-01';
  let hoverPlot: PlotProperties | null = null;
  let shiftDays = 0;
  let drainageImprovePct = 40;
  let theme: 'malam' | 'kertas' = 'kertas';
  type Locale = 'id' | 'en' | 'ja' | 'zh';
  let locale: Locale = 'id';

  const UI_I18N: Record<Locale, Record<string, string>> = {
    id: {
      home: 'Beranda', replay: 'Musim Ulang', drought: 'Kekeringan', methods: 'Metode', lobby: 'Lobby',
      layerYield: 'Hasil (batlow)', layerClay: '% Liat (lajolla)', layerNdvi: 'NDVI (bamako)',
      play: 'Putar Musim', pause: 'Jeda', light: 'Putih', dark: 'Malam',
      tabReplay: 'Replay & Tanah', tabLens: 'Lensa Dangkal vs Dalam', tab3d: 'Blok 3D Tanah', tabByod: 'Lahan Saya (BYOD)',
      copyLink: 'Salin Permalink', copyEmbed: 'Salin Embed', exportGeo: 'Ekspor GeoJSON', bridge: 'Lihat Konteks Kekeringan',
    },
    en: {
      home: 'Home', replay: 'Season Replay', drought: 'Drought', methods: 'Methods', lobby: 'Lobby',
      layerYield: 'Yield (batlow)', layerClay: 'Clay % (lajolla)', layerNdvi: 'NDVI (bamako)',
      play: 'Play Season', pause: 'Pause', light: 'Light', dark: 'Dark',
      tabReplay: 'Replay & Soil', tabLens: 'Shallow vs Deep', tab3d: '3D Soil Block', tabByod: 'My Field (BYOD)',
      copyLink: 'Copy Permalink', copyEmbed: 'Copy Embed', exportGeo: 'Export GeoJSON', bridge: 'Open Drought Context',
    },
    ja: {
      home: 'ホーム', replay: 'リプレイ', drought: '干ばつ', methods: '手法', lobby: 'ロビー',
      layerYield: '収量 (batlow)', layerClay: '粘土率 (lajolla)', layerNdvi: 'NDVI (bamako)',
      play: '再生', pause: '一時停止', light: 'ライト', dark: 'ダーク',
      tabReplay: 'リプレイと土壌', tabLens: '浅層 vs 深層', tab3d: '3D土壌ブロック', tabByod: 'マイ圃場 (BYOD)',
      copyLink: 'URLをコピー', copyEmbed: '埋込コピー', exportGeo: 'GeoJSON出力', bridge: '干ばつ状況を見る',
    },
    zh: {
      home: '首页', replay: '生长季回放', drought: '干旱', methods: '方法', lobby: '大厅',
      layerYield: '产量 (batlow)', layerClay: '黏粒% (lajolla)', layerNdvi: 'NDVI (bamako)',
      play: '播放生长季', pause: '暂停', light: '浅色', dark: '深色',
      tabReplay: '回放与土壤', tabLens: '浅层 vs 深层', tab3d: '三维土壤块', tabByod: '我的农田 (BYOD)',
      copyLink: '复制链接', copyEmbed: '复制嵌入', exportGeo: '导出 GeoJSON', bridge: '查看干旱背景',
    },
  };
  $: tUi = UI_I18N[locale];

  function setLocale(next: Locale) {
    locale = next;
    document.documentElement.lang = next;
    window.localStorage.setItem('agri_locale', next);
  }

  let remainingSec = 300;
  let authenticated = false;
  let userEmail = '';
  let wallActive = false;

  let mapContainer: HTMLDivElement;
  let map: maplibregl.Map | null = null;
  let chartContainer: HTMLDivElement;
  let chart: uPlot | null = null;

  $: activeCrop = CROP_PROFILES[cropKey];
  $: activeDate = bundle?.dates[dayIdx] || '2025-07-15';
  $: activeGdd = wasmGddSeries[dayIdx] ?? bundle?.gdd_cum[dayIdx] ?? 0;
  $: activeStageIdx = wasmStageSeries.length > dayIdx ? wasmStageSeries[dayIdx] : null;
  $: activeStage =
    activeStageIdx !== null
      ? activeCrop.stages[Math.min(5, Math.max(0, Math.round(activeStageIdx)))]
      : bundle?.stage_code[dayIdx] || 'heading';
  $: activeStressed = Boolean(
    bundle?.stress_windows.some((w) => activeDate >= w.start && activeDate <= w.end)
  );
  $: selectedPlot =
    bundle?.plots.features.find((f) => f.properties.plot_id === selectedPlotId)?.properties ||
    bundle?.plots.features[0]?.properties ||
    null;

  $: whatIfYield = (() => {
    if (!selectedPlot || !bundle) return null;
    const base = selectedPlot.yield_t_ha;
    // Heavy-clay penalty recovery via sub-surface/surface drainage improvement
    const clayExcess = Math.max(0, selectedPlot.clay_pct - 40);
    const drainageGain = clayExcess * 0.055 * (drainageImprovePct / 100);
    // Shifting planting date away from ERA5 dry windows around panicle initiation/heading
    const dryDays = bundle.stress_windows?.length ? Math.abs(shiftDays) * 0.024 : Math.abs(shiftDays) * 0.01;
    const shiftGain = shiftDays === 0 ? 0 : Math.min(0.65, dryDays);
    const projected = Math.min(8.2, Number((base + drainageGain + shiftGain).toFixed(2)));
    const delta = Number((projected - base).toFixed(2));
    const pct = base > 0 ? Number(((delta / base) * 100).toFixed(1)) : 0;
    return { base, projected, delta, pct };
  })();

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
    if (mapLayer === 'stability') {
      if (p.stability_class === 'stabil_tinggi') return '#1E6B47';
      if (p.stability_class === 'sensitif_iklim') return '#D97706';
      return '#B83B14';
    }
    if (mapLayer === 'zones') {
      const z = zoneMap[p.plot_id] ?? 1;
      if (z === 2) return '#1E6B47';
      if (z === 1) return '#D97706';
      return '#B83B14';
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
    const activeGddArr = wasmGddSeries.length === bundle.dates.length ? wasmGddSeries : bundle.gdd_cum;
    const gddScaled = activeGddArr.map((g) => Number((g / 40).toFixed(1)));
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
          { label: `GDD ${CROP_PROFILES[cropKey].name}/40`, stroke: '#12703B', width: 1.8 },
        ],
      },
      [xs, bundle.rain_mm, bundle.tmax_c, gddScaled],
      chartContainer
    );
  }

  async function recomputeCropWasm() {
    if (!bundle) return;
    const profile = CROP_PROFILES[cropKey];
    try {
      const gdds = await computeGddSeries(bundle.tmax_c, bundle.tmin_c, profile.tbase, profile.tupper);
      const roundedGdd = gdds.map((g) => Number(g.toFixed(1)));
      wasmGddSeries = roundedGdd;
      if (roundedGdd.length > 0) wasmGddFinal = roundedGdd[roundedGdd.length - 1];
      wasmStageSeries = await computePhenologySeries(roundedGdd, profile.code);
      renderChart();
    } catch {
      wasmGddSeries = [];
      wasmStageSeries = [];
    }
  }

  function handleCropChange(next: CropKey) {
    cropKey = next;
    recomputeCropWasm();
  }

  async function loadField(id: 'bompon-magelang' | 'klaten-irigasi', nextSeason: 'mt2-2025' | 'mt2-2024' = seasonId) {
    fieldId = id;
    seasonId = id === 'klaten-irigasi' ? 'mt2-2025' : nextSeason;
    const file =
      id === 'bompon-magelang'
        ? seasonId === 'mt2-2024'
          ? 'bompon-mt2-2024.json'
          : 'bompon-mt2-2025.json'
        : 'klaten-mt2-2025.json';
    const res = await fetch(`/ulang/mock-data/${file}`);
    bundle = await res.json();
    selectedPlotId = bundle?.plots.features[0]?.properties.plot_id || '';
    dayIdx = Math.min(74, (bundle?.dates.length || 90) - 1);
    await tick();
    if (map && bundle) {
      map.flyTo({ center: bundle.center, zoom: id === 'bompon-magelang' ? 14.1 : 15.2, duration: 700 });
      updateMapPlots();
    }
    renderChart();
    if (bundle) {
      recomputeCropWasm();
      const clays = bundle.plots.features.map((f) => f.properties.clay_pct);
      const ylds = bundle.plots.features.map((f) => f.properties.yield_t_ha);
      const ndvis = bundle.plots.features.map((f) => f.properties.ndvi_peak);
      const perms = bundle.plots.features.map((f) => f.properties.permeability ?? Math.max(2, (100 - f.properties.clay_pct) * 0.4));
      computeOlsSummary(clays, ylds)
        .then((res) => {
          fieldOls = res;
        })
        .catch(() => {});
      Promise.all([
        computeKmeansZones(ylds, clays, ndvis),
        computeYieldExplain(clays, ndvis, perms, ylds),
      ])
        .then(([km, exp]) => {
          const nextMap: Record<string, number> = {};
          bundle?.plots.features.forEach((f, idx) => {
            nextMap[f.properties.plot_id] = km.labels[idx] ?? 1;
          });
          zoneMap = nextMap;
          zoneSep = km.separation;
          yieldExplain = exp;
          updateMapPlots();
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
    u.searchParams.set('crop', cropKey);
    if (selectedPlotId) u.searchParams.set('plot', selectedPlotId);
    u.searchParams.delete('embed');
    return u.toString();
  }

  async function copyPermalink() {
    const urlStr = buildPermalink();
    window.history.replaceState(null, '', urlStr);
    let shareUrl = urlStr;
    let isShort = false;
    try {
      const u = new URL(urlStr);
      const res = await fetch('/api/v1/share', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          module: 'ulang',
          target_path: `${u.pathname}${u.search}`,
        }),
      });
      if (res.ok) {
        const j = await res.json();
        if (j?.short_url) {
          shareUrl = String(j.short_url);
          isShort = true;
        }
      }
    } catch {
      // Fallback safely to full permalink
    }
    try {
      await navigator.clipboard.writeText(shareUrl);
      copyNotice = isShort ? `Tautan singkat tersalin: ${shareUrl}` : 'Permalink tersalin ke papan klip.';
    } catch {
      copyNotice = shareUrl;
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

    const savedLocale = window.localStorage.getItem('agri_locale');
    if (savedLocale === 'id' || savedLocale === 'en' || savedLocale === 'ja' || savedLocale === 'zh') {
      locale = savedLocale;
    }

    const qCrop = qs.get('crop');
    if (qCrop === 'padi' || qCrop === 'jagung' || qCrop === 'kedelai' || qCrop === 'bawang' || qCrop === 'tebu') {
      cropKey = qCrop;
    }
    if (window.location.pathname.includes('klaten')) fieldId = 'klaten-irigasi';
    if (window.location.pathname.includes('lahan-saya')) activeTab = 'lahan';
    await loadField(fieldId);

    const qTab = qs.get('tab');
    if (qTab === 'replay' || qTab === 'analisis' || qTab === 'tanah3d' || qTab === 'lahan') activeTab = qTab;
    const qLayer = qs.get('layer');
    if (qLayer === 'yield' || qLayer === 'clay' || qLayer === 'ndvi' || qLayer === 'stability' || qLayer === 'zones') {
      mapLayer = qLayer;
    }
    const rawDay = qs.get('day');
    if (rawDay !== null && rawDay !== '') {
      const qDay = Number(rawDay);
      if (Number.isInteger(qDay) && bundle && qDay >= 0 && qDay < bundle.dates.length) dayIdx = qDay;
    }
    const qPlot = qs.get('plot');
    if (qPlot && bundle?.plots.features.some((f) => f.properties.plot_id === qPlot)) selectedPlotId = qPlot;

    map = new maplibregl.Map({
      container: mapContainer,
      style: 'https://tiles.openfreemap.org/styles/positron',
      center: bundle?.center || [110.0665, -7.5549],
      zoom: fieldId === 'bompon-magelang' ? 14.1 : 15.2,
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
      map.on('mousemove', 'panen-fill', (e) => {
        if (map) map.getCanvas().style.cursor = 'pointer';
        const props = e.features?.[0]?.properties as PlotProperties | undefined;
        if (props?.plot_id) hoverPlot = props;
      });
      map.on('mouseleave', 'panen-fill', () => {
        if (map) map.getCanvas().style.cursor = '';
        hoverPlot = null;
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
          <a href="/">{tUi.home}</a>
          <a href="/ulang/bompon-magelang/mt2-2025/" class="on">{tUi.replay}</a>
          <a href="/kekeringan/">{tUi.drought}</a>
          <a href="/metode/">{tUi.methods}</a>
          <a href="/data/">Data</a>
          <a href="/lobby/">{tUi.lobby}</a>
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
          {#if fieldId === 'bompon-magelang'}
            <button
              type="button"
              class={seasonId === 'mt2-2025' ? 'btn-primary' : 'btn-subtle'}
              on:click={() => loadField('bompon-magelang', 'mt2-2025')}
            >
              MT2 2025
            </button>
            <button
              type="button"
              class={seasonId === 'mt2-2024' ? 'btn-primary' : 'btn-subtle'}
              on:click={() => loadField('bompon-magelang', 'mt2-2024')}
            >
              MT2 2024 (Kering)
            </button>
          {/if}
          {#if authenticated}
            <span class="live-badge"><span class="live-dot"></span><span>{userEmail}</span></span>
          {:else}
            <a href="/lobby/" class="live-badge">
              <span class={`live-dot ${remainingSec <= 60 ? 'is-warn' : ''}`}></span>
              <span>Pratinjau</span>
              <b>{formatMmSs(remainingSec)}</b>
            </a>
          {/if}
          <select
            aria-label="Bahasa"
            value={locale}
            on:change={(e) => setLocale((e.currentTarget as HTMLSelectElement).value as Locale)}
            class="btn-subtle"
            style="padding: 5px 6px;"
          >
            <option value="id">ID</option>
            <option value="en">EN</option>
            <option value="ja">JA</option>
            <option value="zh">ZH</option>
          </select>
          <button type="button" class="btn-subtle" on:click={toggleTheme}>
            {theme === 'kertas' ? tUi.dark : tUi.light}
          </button>
          <a href="https://github.com/gisrad" target="_blank" rel="noopener noreferrer" class="btn-subtle" style="padding: 5px 7px;" title="GitHub @gisrad" aria-label="GitHub Profil">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
          </a>
          <a href="https://x.com/evansazka" target="_blank" rel="noopener noreferrer" class="btn-subtle" style="padding: 5px 7px;" title="X @evansazka" aria-label="X @evansazka">
            <svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/></svg>
          </a>
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
          {#each (['yield', 'clay', 'ndvi', 'stability', 'zones'] as const) as k}
            <button
              type="button"
              class={mapLayer === k ? 'btn-primary' : 'btn-subtle'}
              style="padding: 6px 9px; font-size: 10px;"
              on:click={() => (mapLayer = k)}
            >
              {k === 'yield' ? tUi.layerYield : k === 'clay' ? tUi.layerClay : k === 'ndvi' ? tUi.layerNdvi : k === 'stability' ? 'Stabilitas 24–25' : 'Zona AI (K-Means)'}
            </button>
          {/each}
        </div>
        {#if hoverPlot}
          <div style="position: absolute; top: 12px; right: 12px; z-index: 10; background: var(--dock-bg); border: 1.5px solid var(--border-ink); padding: 8px 10px; max-width: 240px; pointer-events: none;">
            <div class="num" style="font-size: 10.5px; font-weight: 600;">{hoverPlot.plot_id} · {hoverPlot.zone}</div>
            <div class="num" style="font-size: 10px; color: var(--ink-muted); margin-top: 2px;">
              Hasil {hoverPlot.yield_t_ha} t/ha · Liat {hoverPlot.clay_pct}%{hoverPlot.landslide_class ? ` · ${hoverPlot.landslide_class}` : ''}
            </div>
          </div>
        {/if}
      </div>

      <!-- Scrubber 120 Hari -->
      <div style="border-top: 1px solid var(--border); padding: 12px 16px; background: var(--bg); display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
        <button type="button" class="btn-primary" style="padding: 7px 12px;" on:click={togglePlay}>
          {playing ? tUi.pause : tUi.play}
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
          H+{dayIdx + 1} · {activeDate} · GDD {activeGdd} ({activeCrop.name})
        </span>
      </div>

      <!-- Grafik uPlot -->
      <div style="border-top: 1px solid var(--border); padding: 10px 16px; background: var(--surface);">
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 6px; margin-bottom: 4px;">
          <span class="num" style="font-size: 10px; color: var(--ink-muted);">
            HUJAN HARIAN (BIRU), TMAX °C (TERAKOTA), &amp; AKUMULASI GDD/40 (HIJAU)
          </span>
          <span class="num" style="font-size: 9.5px; color: var(--ink-muted);">
            {bundle?.weather_source || 'Open-Meteo ERA5'}
          </span>
        </div>
        <div bind:this={chartContainer}></div>
      </div>
    </section>

    <!-- Kolom Kanan: Tanaman SVG + 6 Horizon SoilGrids + Lensa Dangkal vs Dalam -->
    <aside style="padding: 18px; display: grid; gap: 16px; align-content: start; overflow-y: auto; max-height: calc(100vh - 90px);">
      <div style="display: flex; gap: 6px; flex-wrap: wrap;">
        <button type="button" class={activeTab === 'replay' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'replay')}>
          {tUi.tabReplay}
        </button>
        <button type="button" class={activeTab === 'analisis' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'analisis')}>
          {tUi.tabLens}
        </button>
        <button type="button" class={activeTab === 'tanah3d' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'tanah3d')}>
          {tUi.tab3d}
        </button>
        <button type="button" class={activeTab === 'lahan' ? 'btn-primary' : 'btn-subtle'} on:click={() => (activeTab = 'lahan')}>
          {tUi.tabByod}
        </button>
      </div>

      {#if wasmGddFinal !== null || fieldOls}
        <div style="border: 1px solid var(--border); padding: 8px 10px; background: var(--surface); display: grid; gap: 6px; font-size: 11px;">
          <div style="display: flex; justify-content: space-between; flex-wrap: wrap; gap: 6px; align-items: center;">
            <span class="num">WASM GDD 120H: {wasmGddFinal !== null ? wasmGddFinal.toFixed(0) : '-'}</span>
            {#if fieldOls}
              <span class="num" style={fieldOls.sampleGuardTriggered ? 'color: #B83B14;' : ''}>
                OLS Liat→Hasil n={fieldOls.n} {fieldOls.sampleGuardTriggered ? '(Guard n<30)' : `R²=${fieldOls.r2.toFixed(2)}`}
              </span>
            {/if}
          </div>
          <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap; border-top: 1px solid var(--border); padding-top: 6px;">
            <label style="display: flex; align-items: center; gap: 6px;">
              <span class="num" style="font-size: 10px; color: var(--ink-muted);">KOMODITAS WASM</span>
              <select
                aria-label="Pilih komoditas fenologi"
                value={cropKey}
                on:change={(e) => handleCropChange((e.currentTarget as HTMLSelectElement).value as CropKey)}
                style="padding: 3px 6px; border: 1px solid var(--border-ink); background: var(--bg); color: var(--ink); font-size: 11px; font-family: inherit;"
              >
                <option value="padi">Padi Sawah</option>
                <option value="jagung">Jagung Hibrida</option>
                <option value="kedelai">Kedelai</option>
                <option value="bawang">Bawang Merah</option>
                <option value="tebu">Tebu</option>
              </select>
            </label>
            <span class="num" style="font-size: 10px; color: var(--ink-muted);">
              Tbase {activeCrop.tbase}°C · Tupper {activeCrop.tupper}°C · Kc {activeCrop.kcMid}
            </span>
          </div>
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
            <CropStageSvg stage={activeStage} stageIndex={activeStageIdx} cropLabel={activeCrop.name} stressed={activeStressed} />
            <div style="display: grid; gap: 6px; align-content: center;">
              <div class="num" style="font-size: 10.5px; color: var(--ink-muted);">HASIL PETAK (SIMULASI)</div>
              <div class="num" style="font-size: 1.6rem; font-weight: 600;">{selectedPlot.yield_t_ha} t/ha</div>
              <div class="num" style="font-size: 11px;">Liat: {selectedPlot.clay_pct}% · NDVI Puncak: {selectedPlot.ndvi_peak}</div>
              {#if selectedPlot.area_ha !== undefined}
                <div class="num" style="font-size: 10.5px; color: var(--ink-muted);">
                  Luas: {selectedPlot.area_ha} ha · Debu/Pasir: {selectedPlot.silt_pct}%/{selectedPlot.sand_pct}% · Perm: {selectedPlot.permeability} mm/j · Longsor: {selectedPlot.landslide_class}
                </div>
              {/if}
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

          {#if yieldExplain}
            {@const feats = bundle.plots.features}
            {@const nf = Math.max(1, feats.length)}
            {@const clays = feats.map((f) => f.properties.clay_pct)}
            {@const ndvis = feats.map((f) => f.properties.ndvi_peak)}
            {@const perms = feats.map((f) => f.properties.permeability ?? Math.max(2, (100 - f.properties.clay_pct) * 0.4))}
            {@const muC = clays.reduce((a, b) => a + b, 0) / nf}
            {@const muN = ndvis.reduce((a, b) => a + b, 0) / nf}
            {@const muP = perms.reduce((a, b) => a + b, 0) / nf}
            {@const sdC = Math.max(1e-6, Math.sqrt(clays.reduce((a, b) => a + (b - muC) * (b - muC), 0) / nf))}
            {@const sdN = Math.max(1e-6, Math.sqrt(ndvis.reduce((a, b) => a + (b - muN) * (b - muN), 0) / nf))}
            {@const sdP = Math.max(1e-6, Math.sqrt(perms.reduce((a, b) => a + (b - muP) * (b - muP), 0) / nf))}
            {@const selP = selectedPlot.permeability ?? Math.max(2, (100 - selectedPlot.clay_pct) * 0.4)}
            {@const dClay = yieldExplain.betas.clay * ((selectedPlot.clay_pct - muC) / sdC)}
            {@const dNdvi = yieldExplain.betas.ndvi * ((selectedPlot.ndvi_peak - muN) / sdN)}
            {@const dPerm = yieldExplain.betas.perm * ((selP - muP) / sdP)}
            {@const zId = zoneMap[selectedPlot.plot_id] ?? 1}
            {@const zLabel = zId === 2 ? 'Zona Potensi Tinggi' : zId === 1 ? 'Zona Transisi' : 'Zona Kritis / Restorasi'}
            <div style="border: 1.5px solid var(--border-ink); padding: 14px; background: var(--bg); display: grid; gap: 10px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 6px; flex-wrap: wrap;">
                <span class="num" style="font-size: 10.5px; font-weight: 600;">EXPLAINABLE AI · RUST WASM</span>
                <span class={`pill-badge ${zId === 0 ? 'pill-warn' : ''}`}>{zLabel}</span>
              </div>

              <div class="num" style="font-size: 10.5px; color: var(--ink-secondary); display: flex; flex-wrap: wrap; gap: 10px;">
                <span>R² Ridge: <strong>{yieldExplain.r2.toFixed(2)}</strong></span>
                <span>RMSE: <strong>{yieldExplain.rmse.toFixed(2)} t/ha</strong></span>
                <span>Silhouette K-Means: <strong>{zoneSep.toFixed(2)}</strong></span>
                {#if yieldExplain.sampleGuardTriggered}
                  <span style="color: #B83B14;">Guard n&lt;30 aktif</span>
                {/if}
              </div>

              <div style="display: grid; gap: 5px;">
                <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">BOBOT FITUR GLOBAL (%)</div>
                {#each [
                  { lbl: 'Tekstur Liat %', pct: yieldExplain.importancePct.clay, col: '#B83B14' },
                  { lbl: 'Kanopi NDVI', pct: yieldExplain.importancePct.ndvi, col: '#1E6B47' },
                  { lbl: 'Permeabilitas', pct: yieldExplain.importancePct.perm, col: '#4E7A96' },
                ] as bar}
                  <div style="display: grid; grid-template-columns: 105px 1fr 44px; gap: 8px; align-items: center; font-size: 11px;">
                    <span class="num">{bar.lbl}</span>
                    <div style="height: 7px; background: var(--border);">
                      <div style={`height: 100%; width: ${Math.min(100, Math.round(bar.pct))}%; background: ${bar.col};`}></div>
                    </div>
                    <span class="num" style="text-align: right;">{bar.pct.toFixed(1)}%</span>
                  </div>
                {/each}
              </div>

              <div style="border-top: 1px solid var(--border); padding-top: 8px; display: grid; gap: 4px;">
                <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">
                  ATRIBUSI LOKAL PETAK VS RATA-RATA LAHAN ({yieldExplain.meanYield.toFixed(2)} t/ha)
                </div>
                <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px; background: var(--border); border: 1px solid var(--border);">
                  <div style="background: var(--surface); padding: 6px 8px;">
                    <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">Δ LIAT</div>
                    <div class="num" style="font-size: 0.98rem; font-weight: 600; color: {dClay >= 0 ? '#1E6B47' : '#B83B14'};">
                      {dClay >= 0 ? '+' : ''}{dClay.toFixed(2)} t/ha
                    </div>
                  </div>
                  <div style="background: var(--surface); padding: 6px 8px;">
                    <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">Δ NDVI</div>
                    <div class="num" style="font-size: 0.98rem; font-weight: 600; color: {dNdvi >= 0 ? '#1E6B47' : '#B83B14'};">
                      {dNdvi >= 0 ? '+' : ''}{dNdvi.toFixed(2)} t/ha
                    </div>
                  </div>
                  <div style="background: var(--surface); padding: 6px 8px;">
                    <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">Δ PERMEABILITAS</div>
                    <div class="num" style="font-size: 0.98rem; font-weight: 600; color: {dPerm >= 0 ? '#1E6B47' : '#B83B14'};">
                      {dPerm >= 0 ? '+' : ''}{dPerm.toFixed(2)} t/ha
                    </div>
                  </div>
                </div>
              </div>
            </div>
          {/if}

          {#if selectedPlot.yield_2024 !== undefined && selectedPlot.yield_2025 !== undefined}
            <div style="border: 1px solid var(--border); padding: 14px; background: var(--bg); display: grid; gap: 10px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; flex-wrap: wrap;">
                <span class="num" style="font-size: 10.5px; font-weight: 600;">STABILITAS LINTAS MUSIM (MT2 2024 VS 2025)</span>
                <span class="pill-badge">{selectedPlot.stability_class || 'sensitif_iklim'}</span>
              </div>
              <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px; background: var(--border); border: 1px solid var(--border);">
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">MT2 2024 (KERING)</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600;">{selectedPlot.yield_2024} t/ha</div>
                </div>
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">MT2 2025 (BASAH)</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600;">{selectedPlot.yield_2025} t/ha</div>
                </div>
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">SELISIH YOY</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600;">+{selectedPlot.yoy_delta_t_ha} t/ha</div>
                </div>
              </div>
            </div>
          {/if}

          {#if whatIfYield}
            <div style="border: 1px solid var(--border); padding: 14px; background: var(--bg); display: grid; gap: 10px;">
              <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px;">
                <span class="num" style="font-size: 10.5px; font-weight: 600;">SIMULATOR WHAT-IF MUSIM DEPAN</span>
                <span class="pill-badge pill-sim">SIMULASI SKENARIO</span>
              </div>
              <label style="display: grid; gap: 4px; font-size: 11px;">
                <span class="num">Geser Tanggal Tanam: {shiftDays >= 0 ? `+${shiftDays}` : shiftDays} hari</span>
                <input type="range" min="-21" max="21" step="1" bind:value={shiftDays} style="accent-color: #000000;" />
              </label>
              <label style="display: grid; gap: 4px; font-size: 11px;">
                <span class="num">Perbaikan Drainase / Pengolahan Zona Liat: {drainageImprovePct}%</span>
                <input type="range" min="0" max="100" step="5" bind:value={drainageImprovePct} style="accent-color: #000000;" />
              </label>
              <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 1px; background: var(--border); border: 1px solid var(--border);">
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">BASIS</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600;">{whatIfYield.base} t/ha</div>
                </div>
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">PROYEKSI</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600;">{whatIfYield.projected} t/ha</div>
                </div>
                <div style="background: var(--surface); padding: 8px;">
                  <div class="num" style="font-size: 9.5px; color: var(--ink-muted);">SELISIH</div>
                  <div class="num" style="font-size: 1.05rem; font-weight: 600; color: #1E6B47;">
                    +{whatIfYield.delta} (+{whatIfYield.pct}%)
                  </div>
                </div>
              </div>
            </div>
          {/if}
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
          <button type="button" class="btn-subtle" on:click={copyPermalink}>{tUi.copyLink}</button>
          <button type="button" class="btn-subtle" on:click={copyEmbed}>{tUi.copyEmbed}</button>
          <button type="button" class="btn-subtle" on:click={exportSubsetGeoJson}>{tUi.exportGeo}</button>
        </div>
        {#if copyNotice}
          <div class="num" style="font-size: 10.5px; color: var(--ink-muted); word-break: break-all;">{copyNotice}</div>
        {/if}

        <a href={`/kekeringan/${bundle.linked_kode_bps}/`} class="btn-outline">
          {tUi.bridge} ({bundle.regency}) →
        </a>
      {/if}
    </aside>
  </main>

  <TrialOverlay bind:remainingSec bind:authenticated bind:userEmail bind:wallActive />
</div>
