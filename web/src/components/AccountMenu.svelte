<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  export let authenticated = false;
  export let userEmail = '';
  export let userName = '';
  export let userAvatar = '';
  export let remainingSec = 300;
  export let nextPath = '/ulang/bompon-magelang/mt2-2025/';

  let open = false;
  let imgFailed = false;
  let wrapEl: HTMLDivElement;

  $: if (userAvatar) {
    imgFailed = false;
  }

  $: initials = (() => {
    const src = (userName || userEmail || 'U').trim();
    const parts = src.split(/\s+/).filter(Boolean);
    if (parts.length >= 2) return `${parts[0][0]}${parts[1][0]}`.toUpperCase();
    return src.slice(0, 2).toUpperCase();
  })();

  $: firstName = (userName || userEmail || 'Pengguna').trim().split(/\s+/)[0];

  function formatMmSs(sec: number): string {
    const s = Math.max(0, Math.floor(sec));
    return `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(s % 60).padStart(2, '0')}`;
  }

  function handleDocClick(e: MouseEvent) {
    if (open && wrapEl && !wrapEl.contains(e.target as Node)) {
      open = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (open && e.key === 'Escape') {
      open = false;
    }
  }

  onMount(() => {
    document.addEventListener('mousedown', handleDocClick);
    document.addEventListener('keydown', handleKeyDown);
  });

  onDestroy(() => {
    if (typeof document !== 'undefined') {
      document.removeEventListener('mousedown', handleDocClick);
      document.removeEventListener('keydown', handleKeyDown);
    }
  });
</script>

{#if authenticated}
  <div class="tb-acc-wrap" bind:this={wrapEl}>
    <button
      type="button"
      class="tb-acc-trigger"
      aria-haspopup="menu"
      aria-expanded={open}
      title={`${userName || 'Pengguna Google'} (${userEmail})`}
      on:click={() => (open = !open)}
    >
      <span class="tb-acc-avatar-box">
        {#if userAvatar && !imgFailed}
          <img
            src={userAvatar}
            alt={userName || userEmail || 'Foto Profil Google'}
            width="24"
            height="24"
            referrerpolicy="no-referrer"
            class="tb-acc-img"
            on:error={() => (imgFailed = true)}
          />
        {:else}
          <span class="tb-acc-fallback" aria-hidden="true">{initials}</span>
        {/if}
        <span class="tb-acc-dot" aria-hidden="true"></span>
      </span>
      <span class="tb-acc-name">{firstName}</span>
      <svg width="10" height="10" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
        <path d="M4.2 5.7 8 9.5l3.8-3.8 1.2 1.2-5 5-5-5z" />
      </svg>
    </button>

    {#if open}
      <div class="tb-acc-popover" role="menu" aria-label="Menu Akun Google">
        <div class="tb-acc-stripe"></div>
        <div class="tb-acc-head">
          {#if userAvatar && !imgFailed}
            <img
              src={userAvatar}
              alt={userName || userEmail || 'Foto Profil Google'}
              width="44"
              height="44"
              referrerpolicy="no-referrer"
              class="tb-acc-img tb-acc-img-lg"
              on:error={() => (imgFailed = true)}
            />
          {:else}
            <span class="tb-acc-fallback tb-acc-fallback-lg" aria-hidden="true">{initials}</span>
          {/if}
          <div style="min-width: 0; flex: 1;">
            <div class="tb-acc-fullname">{userName || 'Pengguna Google'}</div>
            <div class="tb-acc-email num">{userEmail}</div>
            <div class="tb-acc-verified">● Google Terverifikasi</div>
          </div>
        </div>
        <div class="tb-acc-tiers">
          <div>
            <div class="tb-acc-tlbl">AKSES</div>
            <div class="tb-acc-tval">Penuh (7 hari)</div>
          </div>
          <div>
            <div class="tb-acc-tlbl">GEE COLAB</div>
            <div class="tb-acc-tval">2/hari</div>
          </div>
        </div>
        <div class="tb-acc-links">
          <a href="/lobby/" role="menuitem">Lobby &amp; Profil Akun</a>
          <a href="/kekeringan/" role="menuitem">Peta Kekeringan (514 Kab/Kota)</a>
          <a href="/ulang/bompon-magelang/mt2-2025/" role="menuitem">Musim Ulang (Panen-ID)</a>
        </div>
        <form action="/auth/logout" method="post" class="tb-acc-logout-form">
          <button type="submit" class="tb-acc-logout">Keluar Akun Google</button>
        </form>
      </div>
    {/if}
  </div>
{:else}
  <div style="display: inline-flex; align-items: center; gap: 6px;">
    <a href="/lobby/" class="live-badge">
      <span class={`live-dot ${remainingSec <= 60 ? 'is-warn' : ''}`}></span>
      <span>Pratinjau</span>
      <b>{formatMmSs(remainingSec)}</b>
    </a>
    <a
      href={`/auth/google?next=${encodeURIComponent(nextPath)}`}
      class="tb-google-btn"
      title="Masuk dengan Google (gratis, kuota GEE 2/hari)"
    >
      <svg width="13" height="13" viewBox="0 0 24 24" aria-hidden="true">
        <path fill="#4285F4" d="M23.745 12.27c0-.7-.06-1.4-.19-2.07H12v4.51h6.6c-.29 1.52-1.14 2.82-2.4 3.68v3.05h3.88c2.27-2.09 3.665-5.17 3.665-9.17Z" />
        <path fill="#34A853" d="M12 24c3.24 0 5.95-1.08 7.93-2.91l-3.88-3.05c-1.08.72-2.45 1.16-4.05 1.16-3.12 0-5.77-2.11-6.72-4.96H1.29v3.14C3.26 21.3 7.31 24 12 24Z" />
        <path fill="#FBBC05" d="M5.28 14.24c-.24-.72-.38-1.49-.38-2.24s.14-1.52.38-2.24V6.62H1.29C.47 8.24 0 10.06 0 12s.47 3.76 1.29 5.38l3.99-3.14Z" />
        <path fill="#EA4335" d="M12 4.75c1.77 0 3.35.61 4.6 1.8l3.42-3.42C17.95 1.19 15.24 0 12 0 7.31 0 3.26 2.7 1.29 6.62l3.99 3.14c.95-2.85 3.6-4.96 6.72-4.96Z" />
      </svg>
      <span>Masuk</span>
    </a>
  </div>
{/if}

<style>
  .tb-acc-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .tb-acc-trigger {
    appearance: none;
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 3px 9px 3px 3px;
    border-radius: 999px;
    border: 1px solid var(--border-ink);
    background: var(--surface);
    color: var(--ink);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
  }
  .tb-acc-avatar-box {
    position: relative;
    display: inline-flex;
  }
  .tb-acc-img,
  .tb-acc-fallback {
    width: 24px;
    height: 24px;
    border-radius: 999px;
    border: 1.5px solid var(--border-ink);
    object-fit: cover;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: var(--ink);
    color: var(--bg);
    font-family: 'IBM Plex Mono', monospace;
    font-size: 10px;
    font-weight: 600;
  }
  .tb-acc-img-lg,
  .tb-acc-fallback-lg {
    width: 44px;
    height: 44px;
    font-size: 15px;
    flex-shrink: 0;
  }
  .tb-acc-dot {
    position: absolute;
    right: -1px;
    bottom: -1px;
    width: 8px;
    height: 8px;
    border-radius: 999px;
    background: #12703B;
    border: 1.5px solid var(--bg);
  }
  .tb-acc-name {
    max-width: 100px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tb-acc-popover {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    z-index: 60;
    width: min(300px, calc(100vw - 24px));
    background: var(--bg);
    border: 1.5px solid var(--border-ink);
    box-shadow: 0 18px 40px rgba(0, 0, 0, 0.18);
    display: grid;
  }
  .tb-acc-stripe {
    height: 3px;
    background: linear-gradient(90deg, #E8734B 0%, #F3C64B 34%, #7AB88A 68%, #4E7A96 100%);
  }
  .tb-acc-head {
    display: flex;
    align-items: center;
    gap: 11px;
    padding: 12px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
  }
  .tb-acc-fullname {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--ink);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tb-acc-email {
    font-size: 10.5px;
    color: var(--ink-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .tb-acc-verified {
    margin-top: 3px;
    font-family: 'IBM Plex Mono', monospace;
    font-size: 9.5px;
    color: #12703B;
    font-weight: 600;
  }
  .tb-acc-tiers {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1px;
    background: var(--border);
    border-bottom: 1px solid var(--border);
  }
  .tb-acc-tiers > div {
    background: var(--bg);
    padding: 8px 10px;
  }
  .tb-acc-tlbl {
    font-family: 'IBM Plex Mono', monospace;
    font-size: 9px;
    color: var(--ink-muted);
  }
  .tb-acc-tval {
    font-family: 'IBM Plex Mono', monospace;
    font-size: 11px;
    font-weight: 600;
    color: var(--ink);
  }
  .tb-acc-links {
    display: grid;
    padding: 5px;
    border-bottom: 1px solid var(--border);
  }
  .tb-acc-links a {
    padding: 7px 9px;
    text-decoration: none;
    font-size: 12px;
    color: var(--ink-secondary);
  }
  .tb-acc-links a:hover {
    background: var(--surface);
    color: var(--ink);
  }
  .tb-acc-logout-form {
    margin: 0;
    padding: 8px;
  }
  .tb-acc-logout {
    appearance: none;
    width: 100%;
    padding: 7px 10px;
    border: 1px solid var(--border);
    background: transparent;
    color: #B83B14;
    font-family: 'IBM Plex Mono', monospace;
    font-size: 10.5px;
    font-weight: 600;
    text-transform: uppercase;
    cursor: pointer;
  }
  .tb-google-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 1px solid var(--border-ink);
    background: var(--bg);
    color: var(--ink);
    font-family: 'IBM Plex Mono', monospace;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    text-decoration: none;
  }
  .tb-google-btn:hover {
    background: var(--ink);
    color: var(--bg);
  }
</style>
