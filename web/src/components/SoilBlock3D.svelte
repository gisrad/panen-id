<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export interface SoilHorizon {
    depth_cm: string;
    clay_pct: number;
    ph_h2o: number;
    soc_g_kg?: number;
  }

  export interface PlotFeature {
    properties: {
      plot_id: string;
      zone: string;
      yield_t_ha: number;
      clay_pct: number;
      drainage?: string;
    };
  }

  export let soilProfile: SoilHorizon[] = [];
  export let plots: PlotFeature[] = [];
  export let selectedPlotId = '';
  export let fieldName = 'DAS Bompon';

  let canvasWrap: HTMLDivElement;
  let status = 'Memuat modul WebGL 3D (lazy)...';
  let errorMsg = '';
  let cleanupFn: (() => void) | null = null;

  onMount(async () => {
    try {
      const THREE = await import('three');
      if (!canvasWrap) return;

      const width = Math.max(320, canvasWrap.clientWidth || 440);
      const height = 280;

      const scene = new THREE.Scene();
      scene.background = new THREE.Color('#FAFAFA');

      const camera = new THREE.PerspectiveCamera(40, width / height, 0.1, 100);
      let azimuth = 0.78;
      let elevation = 0.56;
      let radius = 11.2;

      const updateCamera = () => {
        camera.position.set(
          radius * Math.cos(elevation) * Math.sin(azimuth),
          radius * Math.sin(elevation),
          radius * Math.cos(elevation) * Math.cos(azimuth),
        );
        camera.lookAt(0, -0.35, 0);
      };
      updateCamera();

      const renderer = new THREE.WebGLRenderer({ antialias: true });
      renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
      renderer.setSize(width, height);
      canvasWrap.innerHTML = '';
      canvasWrap.appendChild(renderer.domElement);

      const hemi = new THREE.HemisphereLight('#ffffff', '#3d352e', 1.15);
      scene.add(hemi);
      const dir = new THREE.DirectionalLight('#fff8eb', 0.85);
      dir.position.set(6, 10, 5);
      scene.add(dir);

      const group = new THREE.Group();
      scene.add(group);

      // 1. Top surface terrain/yield relief (5 x 5 block, subdivided)
      const topW = 5.2;
      const topD = 5.2;
      const topGeo = new THREE.PlaneGeometry(topW, topD, 24, 24);
      topGeo.rotateX(-Math.PI / 2);

      const pos = topGeo.attributes.position;
      const colors = new Float32Array(pos.count * 3);
      const cLow = new THREE.Color('#B83B14');
      const cMid = new THREE.Color('#F3C64B');
      const cHigh = new THREE.Color('#1E6B47');
      const tmp = new THREE.Color();

      for (let i = 0; i < pos.count; i++) {
        const vx = pos.getX(i);
        const vz = pos.getZ(i);
        // Synthetic hillslope + plot variation for visual soil block relief
        const idx = Math.min(
          plots.length - 1,
          Math.max(0, Math.floor(((vx + topW / 2) / topW) * 3) + Math.floor(((vz + topD / 2) / topD) * 3) * 3),
        );
        const yieldVal = plots[idx]?.properties?.yield_t_ha ?? 5.5;
        const t = Math.min(1, Math.max(0, (yieldVal - 3.2) / 4.2));
        const relief = 0.28 * Math.sin(vx * 0.65) + 0.14 * Math.cos(vz * 0.7) + (t - 0.5) * 0.25;
        pos.setY(i, relief);

        if (t < 0.5) {
          tmp.copy(cLow).lerp(cMid, t * 2);
        } else {
          tmp.copy(cMid).lerp(cHigh, (t - 0.5) * 2);
        }
        colors[i * 3] = tmp.r;
        colors[i * 3 + 1] = tmp.g;
        colors[i * 3 + 2] = tmp.b;
      }
      topGeo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
      topGeo.computeVertexNormals();

      const topMat = new THREE.MeshStandardMaterial({
        vertexColors: true,
        roughness: 0.75,
        metalness: 0.05,
        side: THREE.DoubleSide,
      });
      const topMesh = new THREE.Mesh(topGeo, topMat);
      group.add(topMesh);

      // Hairline wireframe overlay on surface
      const wireMat = new THREE.MeshBasicMaterial({
        color: '#000000',
        wireframe: true,
        transparent: true,
        opacity: 0.12,
      });
      group.add(new THREE.Mesh(topGeo, wireMat));

      // 2. Six stratigraphic soil horizons (depth 0 -> -2.1 units)
      const horizons =
        soilProfile.length > 0
          ? soilProfile
          : [
              { depth_cm: '0-5', clay_pct: 38, ph_h2o: 5.7 },
              { depth_cm: '5-15', clay_pct: 42, ph_h2o: 5.6 },
              { depth_cm: '15-30', clay_pct: 49, ph_h2o: 5.5 },
              { depth_cm: '30-60', clay_pct: 58, ph_h2o: 5.4 },
              { depth_cm: '60-100', clay_pct: 63, ph_h2o: 5.3 },
              { depth_cm: '100-200', clay_pct: 56, ph_h2o: 5.4 },
            ];

      const thicknessWeights = [0.16, 0.22, 0.28, 0.38, 0.48, 0.62];
      let yTop = -0.18;
      horizons.forEach((h, idx) => {
        const hThick = thicknessWeights[idx] ?? 0.35;
        const boxGeo = new THREE.BoxGeometry(topW, hThick, topD);
        // More clay = deeper terracotta-brown tone
        const clayFactor = Math.min(1, Math.max(0, (h.clay_pct - 25) / 45));
        const col = new THREE.Color('#C9A47B').lerp(new THREE.Color('#7B3618'), clayFactor);
        const mat = new THREE.MeshStandardMaterial({
          color: col,
          roughness: 0.9,
          metalness: 0.02,
        });
        const mesh = new THREE.Mesh(boxGeo, mat);
        mesh.position.y = yTop - hThick / 2;
        group.add(mesh);

        const edges = new THREE.EdgesGeometry(boxGeo);
        const line = new THREE.LineSegments(
          edges,
          new THREE.LineBasicMaterial({ color: '#111111', transparent: true, opacity: 0.55 }),
        );
        line.position.y = mesh.position.y;
        group.add(line);

        yTop -= hThick;
      });

      // Pointer drag & wheel zoom
      let dragging = false;
      let lastX = 0;
      let lastY = 0;
      const dom = renderer.domElement;

      const onDown = (e: PointerEvent) => {
        dragging = true;
        lastX = e.clientX;
        lastY = e.clientY;
        dom.setPointerCapture(e.pointerId);
      };
      const onMove = (e: PointerEvent) => {
        if (!dragging) return;
        const dx = e.clientX - lastX;
        const dy = e.clientY - lastY;
        lastX = e.clientX;
        lastY = e.clientY;
        azimuth -= dx * 0.008;
        elevation = Math.min(1.25, Math.max(0.18, elevation + dy * 0.006));
        updateCamera();
      };
      const onUp = () => {
        dragging = false;
      };
      const onWheel = (e: WheelEvent) => {
        e.preventDefault();
        radius = Math.min(18, Math.max(7.2, radius + e.deltaY * 0.006));
        updateCamera();
      };
      const onResize = () => {
        if (!canvasWrap) return;
        const w = Math.max(320, canvasWrap.clientWidth || 440);
        camera.aspect = w / height;
        camera.updateProjectionMatrix();
        renderer.setSize(w, height);
      };

      dom.addEventListener('pointerdown', onDown);
      dom.addEventListener('pointermove', onMove);
      dom.addEventListener('pointerup', onUp);
      dom.addEventListener('wheel', onWheel, { passive: false });
      window.addEventListener('resize', onResize);

      let rafId = 0;
      const renderLoop = () => {
        if (!dragging) {
          azimuth += 0.0014;
          updateCamera();
        }
        renderer.render(scene, camera);
        rafId = requestAnimationFrame(renderLoop);
      };
      renderLoop();
      status = `Blok 3D ${fieldName} aktif — seret untuk putar, gulir untuk zoom`;

      cleanupFn = () => {
        cancelAnimationFrame(rafId);
        dom.removeEventListener('pointerdown', onDown);
        dom.removeEventListener('pointermove', onMove);
        dom.removeEventListener('pointerup', onUp);
        dom.removeEventListener('wheel', onWheel);
        window.removeEventListener('resize', onResize);
        group.traverse((obj: any) => {
          if (obj.geometry) obj.geometry.dispose();
          if (obj.material) {
            if (Array.isArray(obj.material)) obj.material.forEach((m: any) => m.dispose());
            else obj.material.dispose();
          }
        });
        renderer.dispose();
      };
    } catch (err) {
      errorMsg = err instanceof Error ? err.message : 'Gagal inisialisasi Three.js WebGL';
    }
  });

  onDestroy(() => {
    cleanupFn?.();
  });
</script>

<div style="border: 1px solid var(--border); background: var(--surface); display: grid;">
  <div style="display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border-bottom: 1px solid var(--border);">
    <span class="num" style="font-size: 10.5px; color: var(--ink-muted);">
      BLOK STRATIGRAFI TANAH 3D (6 HORIZON 0–200 CM) · PETAK {selectedPlotId || 'UMUM'}
    </span>
    <span class="pill-badge">LAZY THREE.JS</span>
  </div>

  {#if errorMsg}
    <div style="padding: 18px; color: #B83B14; font-size: 0.9rem;">{errorMsg}</div>
  {:else}
    <div bind:this={canvasWrap} style="width: 100%; height: 280px; cursor: grab; overflow: hidden;"></div>
  {/if}

  <div style="padding: 8px 12px; border-top: 1px solid var(--border); display: flex; justify-content: space-between; flex-wrap: wrap; gap: 8px; font-size: 11px;">
    <span class="num" style="color: var(--ink-muted);">{status}</span>
    <span class="num">Permukaan: Hasil (t/ha) · Dinding: Liat SoilGrids (%)</span>
  </div>
</div>
