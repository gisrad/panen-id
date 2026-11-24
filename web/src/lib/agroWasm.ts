export interface AgroWasmExports extends WebAssembly.Exports {
  memory: WebAssembly.Memory;
  alloc_f64: (len: number) => number;
  dealloc_f64: (ptr: number, len: number) => void;
  wasm_gdd_series: (
    tmaxPtr: number,
    tminPtr: number,
    len: number,
    tbase: number,
    tupper: number,
    outPtr: number,
  ) => void;
  wasm_fao56_et0_series: (
    tmaxPtr: number,
    tminPtr: number,
    rhPtr: number,
    u2Ptr: number,
    rsPtr: number,
    doyPtr: number,
    len: number,
    latDeg: number,
    elevM: number,
    outPtr: number,
  ) => void;
  wasm_ols_summary: (xPtr: number, yPtr: number, len: number, outPtr: number) => void;
  wasm_idw_grid: (
    ptsX: number,
    ptsY: number,
    ptsVal: number,
    nPts: number,
    power: number,
    gridX: number,
    gridY: number,
    nGrid: number,
    outPtr: number,
  ) => void;
  wasm_phenology_series: (gddPtr: number, len: number, cropCode: number, outPtr: number) => void;
  wasm_kmeans_zones: (
    yieldsPtr: number,
    claysPtr: number,
    ndvisPtr: number,
    len: number,
    outLabelsPtr: number,
  ) => number;
  wasm_explain_yield: (
    claysPtr: number,
    ndvisPtr: number,
    permsPtr: number,
    yieldsPtr: number,
    len: number,
    out10Ptr: number,
  ) => void;
}

export interface YieldExplainResult {
  betas: { clay: number; ndvi: number; perm: number };
  importancePct: { clay: number; ndvi: number; perm: number };
  r2: number;
  rmse: number;
  sampleGuardTriggered: boolean;
  meanYield: number;
  n: number;
}

export type CropKey = 'padi' | 'jagung' | 'kedelai' | 'bawang' | 'tebu';

export interface CropProfile {
  code: number;
  name: string;
  latin: string;
  tbase: number;
  tupper: number;
  kcMid: number;
  stages: [string, string, string, string, string, string];
}

export const CROP_PROFILES: Record<CropKey, CropProfile> = {
  padi: {
    code: 0,
    name: 'Padi Sawah',
    latin: 'Oryza sativa',
    tbase: 10,
    tupper: 35,
    kcMid: 1.2,
    stages: ['bibit', 'anakan', 'bunting', 'heading', 'pengisian', 'masak'],
  },
  jagung: {
    code: 1,
    name: 'Jagung Hibrida',
    latin: 'Zea mays',
    tbase: 10,
    tupper: 34,
    kcMid: 1.2,
    stages: ['VE', 'V6', 'VT', 'R1', 'R4', 'R6'],
  },
  kedelai: {
    code: 2,
    name: 'Kedelai',
    latin: 'Glycine max',
    tbase: 10,
    tupper: 33,
    kcMid: 1.15,
    stages: ['VE', 'V3', 'R1', 'R3', 'R5', 'R8'],
  },
  bawang: {
    code: 3,
    name: 'Bawang Merah',
    latin: 'Allium cepa',
    tbase: 7,
    tupper: 30,
    kcMid: 1.05,
    stages: ['tunas', 'daun', 'umbi-awal', 'pembesaran', 'pematangan', 'panen'],
  },
  tebu: {
    code: 4,
    name: 'Tebu',
    latin: 'Saccharum officinarum',
    tbase: 12,
    tupper: 38,
    kcMid: 1.25,
    stages: ['kecambah', 'anakan', 'pemanjangan', 'batang-aktif', 'kemasakan', 'tebang'],
  },
};

export interface OlsSummary {
  slope: number;
  intercept: number;
  r2: number;
  sampleGuardTriggered: boolean;
  n: number;
}

let wasmPromise: Promise<AgroWasmExports> | null = null;

export function loadAgroWasm(wasmUrl = '/ulang/wasm/agro_core.wasm'): Promise<AgroWasmExports> {
  if (wasmPromise) return wasmPromise;
  wasmPromise = (async () => {
    const res = await fetch(wasmUrl);
    if (!res.ok) {
      throw new Error(`Gagal memuat agro_core.wasm (${res.status})`);
    }
    let instance: WebAssembly.Instance;
    if (typeof WebAssembly.instantiateStreaming === 'function') {
      try {
        const cloned = res.clone();
        const instantiated = await WebAssembly.instantiateStreaming(cloned, {});
        instance = instantiated.instance;
      } catch {
        const bytes = await res.arrayBuffer();
        const instantiated = await WebAssembly.instantiate(bytes, {});
        instance = instantiated.instance;
      }
    } else {
      const bytes = await res.arrayBuffer();
      const instantiated = await WebAssembly.instantiate(bytes, {});
      instance = instantiated.instance;
    }
    return instance.exports as AgroWasmExports;
  })();
  return wasmPromise;
}

function writeF64(wasm: AgroWasmExports, values: number[]): number {
  const ptr = wasm.alloc_f64(values.length);
  const view = new Float64Array(wasm.memory.buffer, ptr, values.length);
  view.set(values);
  return ptr;
}

function readF64(wasm: AgroWasmExports, ptr: number, len: number): number[] {
  const view = new Float64Array(wasm.memory.buffer, ptr, len);
  return Array.from(view);
}

export async function computeGddSeries(
  tmax: number[],
  tmin: number[],
  tbase = 10,
  tupper = 35,
): Promise<number[]> {
  const n = Math.min(tmax.length, tmin.length);
  if (n === 0) return [];
  const wasm = await loadAgroWasm();
  const pMax = writeF64(wasm, tmax.slice(0, n));
  const pMin = writeF64(wasm, tmin.slice(0, n));
  const pOut = wasm.alloc_f64(n);
  try {
    wasm.wasm_gdd_series(pMax, pMin, n, tbase, tupper, pOut);
    return readF64(wasm, pOut, n);
  } finally {
    wasm.dealloc_f64(pMax, n);
    wasm.dealloc_f64(pMin, n);
    wasm.dealloc_f64(pOut, n);
  }
}

export async function computeFao56Series(
  tmax: number[],
  tmin: number[],
  rh: number[],
  u2: number[],
  rs: number[],
  doy: number[],
  latDeg: number,
  elevM: number,
): Promise<number[]> {
  const n = Math.min(tmax.length, tmin.length, rh.length, u2.length, rs.length, doy.length);
  if (n === 0) return [];
  const wasm = await loadAgroWasm();
  const pTmax = writeF64(wasm, tmax.slice(0, n));
  const pTmin = writeF64(wasm, tmin.slice(0, n));
  const pRh = writeF64(wasm, rh.slice(0, n));
  const pU2 = writeF64(wasm, u2.slice(0, n));
  const pRs = writeF64(wasm, rs.slice(0, n));
  const pDoy = writeF64(wasm, doy.slice(0, n));
  const pOut = wasm.alloc_f64(n);
  try {
    wasm.wasm_fao56_et0_series(pTmax, pTmin, pRh, pU2, pRs, pDoy, n, latDeg, elevM, pOut);
    return readF64(wasm, pOut, n);
  } finally {
    wasm.dealloc_f64(pTmax, n);
    wasm.dealloc_f64(pTmin, n);
    wasm.dealloc_f64(pRh, n);
    wasm.dealloc_f64(pU2, n);
    wasm.dealloc_f64(pRs, n);
    wasm.dealloc_f64(pDoy, n);
    wasm.dealloc_f64(pOut, n);
  }
}

export async function computeOlsSummary(x: number[], y: number[]): Promise<OlsSummary> {
  const n = Math.min(x.length, y.length);
  if (n === 0) {
    return { slope: 0, intercept: 0, r2: 0, sampleGuardTriggered: true, n: 0 };
  }
  const wasm = await loadAgroWasm();
  const px = writeF64(wasm, x.slice(0, n));
  const py = writeF64(wasm, y.slice(0, n));
  const pout = wasm.alloc_f64(4);
  try {
    wasm.wasm_ols_summary(px, py, n, pout);
    const [slope, intercept, r2, guard] = readF64(wasm, pout, 4);
    return {
      slope,
      intercept,
      r2,
      sampleGuardTriggered: guard >= 0.5 || n < 30,
      n,
    };
  } finally {
    wasm.dealloc_f64(px, n);
    wasm.dealloc_f64(py, n);
    wasm.dealloc_f64(pout, 4);
  }
}

export async function computeIdwGrid(
  ptsX: number[],
  ptsY: number[],
  ptsVal: number[],
  gridX: number[],
  gridY: number[],
  power = 2.0,
): Promise<number[]> {
  const nPts = Math.min(ptsX.length, ptsY.length, ptsVal.length);
  const nGrid = Math.min(gridX.length, gridY.length);
  if (nPts === 0 || nGrid === 0) return [];
  const wasm = await loadAgroWasm();
  const pX = writeF64(wasm, ptsX.slice(0, nPts));
  const pY = writeF64(wasm, ptsY.slice(0, nPts));
  const pV = writeF64(wasm, ptsVal.slice(0, nPts));
  const gX = writeF64(wasm, gridX.slice(0, nGrid));
  const gY = writeF64(wasm, gridY.slice(0, nGrid));
  const pOut = wasm.alloc_f64(nGrid);
  try {
    wasm.wasm_idw_grid(pX, pY, pV, nPts, power, gX, gY, nGrid, pOut);
    return readF64(wasm, pOut, nGrid);
  } finally {
    wasm.dealloc_f64(pX, nPts);
    wasm.dealloc_f64(pY, nPts);
    wasm.dealloc_f64(pV, nPts);
    wasm.dealloc_f64(gX, nGrid);
    wasm.dealloc_f64(gY, nGrid);
    wasm.dealloc_f64(pOut, nGrid);
  }
}

export async function computePhenologySeries(gddCum: number[], cropCode: number): Promise<number[]> {
  const n = gddCum.length;
  if (n === 0) return [];
  const wasm = await loadAgroWasm();
  const pGdd = writeF64(wasm, gddCum);
  const pOut = wasm.alloc_f64(n);
  try {
    wasm.wasm_phenology_series(pGdd, n, cropCode, pOut);
    return readF64(wasm, pOut, n);
  } finally {
    wasm.dealloc_f64(pGdd, n);
    wasm.dealloc_f64(pOut, n);
  }
}

export async function computeKmeansZones(
  yields: number[],
  clays: number[],
  ndvis: number[],
): Promise<{ labels: number[]; separation: number }> {
  const n = Math.min(yields.length, clays.length, ndvis.length);
  if (n === 0) return { labels: [], separation: 0 };
  const wasm = await loadAgroWasm();
  const pY = writeF64(wasm, yields.slice(0, n));
  const pC = writeF64(wasm, clays.slice(0, n));
  const pN = writeF64(wasm, ndvis.slice(0, n));
  const pOut = wasm.alloc_f64(n);
  try {
    const separation = wasm.wasm_kmeans_zones(pY, pC, pN, n, pOut);
    return { labels: readF64(wasm, pOut, n).map((v) => Math.round(v)), separation };
  } finally {
    wasm.dealloc_f64(pY, n);
    wasm.dealloc_f64(pC, n);
    wasm.dealloc_f64(pN, n);
    wasm.dealloc_f64(pOut, n);
  }
}

export async function computeYieldExplain(
  clays: number[],
  ndvis: number[],
  perms: number[],
  yields: number[],
): Promise<YieldExplainResult> {
  const n = Math.min(clays.length, ndvis.length, perms.length, yields.length);
  if (n === 0) {
    return {
      betas: { clay: 0, ndvi: 0, perm: 0 },
      importancePct: { clay: 33.4, ndvi: 33.3, perm: 33.3 },
      r2: 0,
      rmse: 0,
      sampleGuardTriggered: true,
      meanYield: 0,
      n: 0,
    };
  }
  const wasm = await loadAgroWasm();
  const pC = writeF64(wasm, clays.slice(0, n));
  const pN = writeF64(wasm, ndvis.slice(0, n));
  const pP = writeF64(wasm, perms.slice(0, n));
  const pY = writeF64(wasm, yields.slice(0, n));
  const pOut = wasm.alloc_f64(10);
  try {
    wasm.wasm_explain_yield(pC, pN, pP, pY, n, pOut);
    const [b1, b2, b3, ic, iN, ip, r2, rmse, guard, meanYield] = readF64(wasm, pOut, 10);
    return {
      betas: { clay: b1, ndvi: b2, perm: b3 },
      importancePct: { clay: ic, ndvi: iN, perm: ip },
      r2,
      rmse,
      sampleGuardTriggered: guard >= 0.5 || n < 30,
      meanYield,
      n,
    };
  } finally {
    wasm.dealloc_f64(pC, n);
    wasm.dealloc_f64(pN, n);
    wasm.dealloc_f64(pP, n);
    wasm.dealloc_f64(pY, n);
    wasm.dealloc_f64(pOut, 10);
  }
}
