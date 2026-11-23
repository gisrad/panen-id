export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    let p = url.pathname;

    if (p === '/ulang') {
      p = '/';
    } else if (p.startsWith('/ulang/')) {
      p = p.slice('/ulang'.length);
    } else if (p === '/analisis' || p.startsWith('/analisis/') || p === '/lahan-saya' || p.startsWith('/lahan-saya/')) {
      p = '/';
    }

    const assetUrl = new URL(p + url.search, url.origin);
    let res = await env.ASSETS.fetch(new Request(assetUrl.toString(), request));

    const hasExt = /\.[a-zA-Z0-9]{1,8}$/.test(p);
    if (res.status === 404 && !hasExt) {
      const idxUrl = new URL('/index.html', url.origin);
      res = await env.ASSETS.fetch(new Request(idxUrl.toString(), request));
    }

    return res;
  },
};
