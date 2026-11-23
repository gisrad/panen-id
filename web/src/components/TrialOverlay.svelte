<script lang="ts">
  import { onMount } from 'svelte';

  export let remainingSec = 300;
  export let authenticated = false;
  export let userEmail = '';
  export let wallActive = false;

  let forceTestWall = false;
  let expiresAtMs = Date.now() + 300_000;
  let serverOffsetMs = 0;

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    if (params.get('trial') === 'expired') {
      forceTestWall = true;
      wallActive = true;
    }

    fetch('/api/auth/session', { credentials: 'same-origin' })
      .then((r) => (r.ok ? r.json() : null))
      .then((data) => {
        if (!data) return;
        authenticated = Boolean(data.authenticated);
        userEmail = data.user?.email || '';
        if (!authenticated && typeof data.trialExpiresAtMs === 'number') {
          expiresAtMs = data.trialExpiresAtMs;
          serverOffsetMs = (data.serverNowMs || Date.now()) - Date.now();
          remainingSec = Math.max(0, Math.ceil((expiresAtMs - (Date.now() + serverOffsetMs)) / 1000));
          if (remainingSec <= 0) wallActive = true;
        }
      })
      .catch(() => {});

    const iv = window.setInterval(() => {
      if (authenticated) return;
      remainingSec = Math.max(0, Math.ceil((expiresAtMs - (Date.now() + serverOffsetMs)) / 1000));
      if (remainingSec <= 0 || forceTestWall) wallActive = true;
    }, 1000);

    const onKey = (e: KeyboardEvent) => {
      if (wallActive && e.key === 'Escape') {
        e.preventDefault();
        e.stopPropagation();
      }
    };
    window.addEventListener('keydown', onKey, true);
    return () => {
      window.clearInterval(iv);
      window.removeEventListener('keydown', onKey, true);
    };
  });

  $: currentNext =
    typeof window !== 'undefined'
      ? `${window.location.pathname}${window.location.search}`
      : '/ulang/bompon-magelang/mt2-2025/';
</script>

{#if wallActive}
  <div class="auth-wall-backdrop" role="dialog" aria-modal="true" aria-labelledby="panen-wall-title">
    <div class="auth-wall-card">
      <div style="margin-bottom: 10px;">
        <span class="pill-badge pill-warn">TRIAL_EXPIRED · 05:00</span>
      </div>
      <h2 id="panen-wall-title" style="margin: 0 0 10px; font-size: 1.55rem; font-weight: 600;">
        Waktu Pratinjau Publik 5 Menit Telah Habis
      </h2>
      <p style="margin: 0 0 16px; color: var(--ink-secondary); font-size: 0.92rem;">
        Masuk gratis dengan Google untuk melanjutkan inspeksi Musim Ulang &amp; Lensa Lingkungan, atau kembali ke Lobby.
      </p>
      <div style="padding: 9px 11px; border: 1px solid var(--border); background: var(--surface); margin-bottom: 18px; font-size: 0.8rem;">
        <span style="color: var(--ink-muted);">Tujuan tersimpan: </span>
        <code class="num">{currentNext}</code>
      </div>
      <div style="display: grid; gap: 10px;">
        <a class="btn-primary" href={`/auth/google?next=${encodeURIComponent(currentNext)}`}>
          Lanjut dengan Google (Gratis)
        </a>
        <a class="btn-outline" href={`/lobby/?reason=trial_expired&next=${encodeURIComponent(currentNext)}`}>
          Kembali ke Lobby
        </a>
      </div>
    </div>
  </div>
{/if}
