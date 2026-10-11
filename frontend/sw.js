const CACHE = 'court-v4';
const ASSETS = [
  '/',
  '/index.html',
  '/local.html',
  '/game.html',
  '/loading.html',
  '/review.html',
  '/css/review.css',
  '/js/review.js',
  '/css/themes.css',
  '/css/board.css',
  '/js/board.js',
  '/js/selection.js',
  '/js/rules.js',
  '/js/local.js',
  '/room.html',
  '/css/room.css',
  '/js/room.js',
  '/css/summon.css',
  '/js/summon.js',
  '/js/game.js',
  '/js/websocket.js',
  '/js/theme.js',
  '/manifest.json',
  '/assets/portraits/crane.jpeg',
  '/assets/portraits/crow.jpeg',
  '/assets/portraits/dragon.jpeg',
  '/assets/portraits/eagle.jpeg',
  '/assets/portraits/jaguar_warm.jpeg',
  '/assets/portraits/jaguar_cold.jpeg',
  '/assets/portraits/lion.jpeg',
  '/assets/portraits/mantis_shrimp.jpeg',
  '/assets/portraits/praying_mantis.jpeg',
  '/assets/portraits/spider.jpeg',
];

self.addEventListener('install', (e) => {
  e.waitUntil(caches.open(CACHE).then(c => c.addAll(ASSETS)));
  self.skipWaiting();
});

self.addEventListener('activate', (e) => {
  e.waitUntil(
    caches.keys().then(keys =>
      Promise.all(keys.filter(k => k !== CACHE).map(k => caches.delete(k)))
    )
  );
  self.clients.claim();
});

self.addEventListener('fetch', (e) => {
  // Only same-origin GETs: let WebSockets, the pod, /summon and /status pass through
  const url = new URL(e.request.url);
  if (e.request.method !== 'GET' || url.origin !== self.location.origin) return;
  if (url.pathname === '/ws' || url.pathname === '/status') return;

  // Network first, so HTML/JS are never stale; the cache is only an offline fallback
  e.respondWith(
    fetch(e.request).catch(() => caches.match(e.request))
  );
});
