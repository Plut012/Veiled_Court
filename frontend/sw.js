const CACHE = 'court-v1';
const ASSETS = [
  '/',
  '/index.html',
  '/local.html',
  '/game.html',
  '/css/themes.css',
  '/css/board.css',
  '/js/board.js',
  '/js/selection.js',
  '/js/local.js',
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
  // Let WebSocket requests pass through
  if (e.request.url.includes('/ws')) return;

  e.respondWith(
    fetch(e.request).catch(() => caches.match(e.request))
  );
});
