// Friend rooms: create a room from the selection screen, play in room.html
// Rules come from rules.js (goEngine); the coordinator only relays moves.

const CLIENT_ID_KEY = 'court-client-id';
let memoryClientId = null;

function newClientId() {
  // getRandomValues works on plain http too (randomUUID does not)
  const bytes = new Uint8Array(16);
  crypto.getRandomValues(bytes);
  return Array.from(bytes, b => b.toString(16).padStart(2, '0')).join('');
}

// Stable anonymous id so a refresh or reconnect gets the same seat
function roomClientId() {
  for (const getStore of [() => localStorage, () => sessionStorage]) {
    try {
      const store = getStore();
      let id = store.getItem(CLIENT_ID_KEY);
      if (!id) {
        id = newClientId();
        store.setItem(CLIENT_ID_KEY, id);
      }
      return id;
    } catch (e) {
      // Storage blocked (private mode etc.) — try the next one
    }
  }
  if (!memoryClientId) memoryClientId = newClientId();
  return memoryClientId;
}

async function createRoom(size) {
  const res = await fetch('/room', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ size, client_id: roomClientId() }),
  });
  const data = await res.json().catch(() => ({}));
  if (!res.ok || !data.code) throw new Error(data.error || 'the court is unreachable');
  window.location.href = `/room/${data.code}`;
}

// --- Selection screen entry ---

function initFriendEntry() {
  const toggle = document.getElementById('friend-toggle');
  const sizes = document.getElementById('friend-sizes');
  const error = document.getElementById('friend-error');

  toggle.addEventListener('click', () => {
    sizes.hidden = !sizes.hidden;
  });

  sizes.querySelectorAll('.friend-size').forEach(btn => {
    btn.addEventListener('click', () => {
      error.textContent = '';
      createRoom(parseInt(btn.dataset.size)).catch(e => {
        error.textContent = e.message;
      });
    });
  });
}

// --- Room page ---

function initRoomPage() {
  const match = window.location.pathname.match(/^\/room\/([a-z0-9]+)\/?$/);
  const code = match ? match[1] : null;
  const clientId = roomClientId();

  const waitPanel = document.getElementById('room-wait');
  const missingPanel = document.getElementById('room-missing');
  const gameScreen = document.getElementById('game-screen');
  const shareBtn = document.getElementById('share-link');
  const shareHint = document.getElementById('share-hint');
  const statusEl = document.getElementById('room-status');
  const controls = document.getElementById('local-controls');
  const passBtn = document.getElementById('pass-btn');
  const resignBtn = document.getElementById('resign-btn');
  const submitTop = document.getElementById('submit-bar-top');
  const submitBottom = document.getElementById('submit-bar-bottom');
  const canvas = document.getElementById('board');

  let ws = null;
  let retries = 0;
  let missing = false;
  let boardReady = false;
  let previewCoords = null;
  let resignTimer = null;

  // Server state, mirrored
  let you = null;          // 'black' | 'white' | 'spectator'
  let size = null;
  let applied = 0;         // moves applied to goEngine
  let result = null;
  let seated = { black: false, white: false };
  let connected = { black: false, white: false };

  const colorNum = { black: 1, white: 2 };

  function showMissing() {
    missing = true;
    if (ws) ws.close();
    waitPanel.hidden = true;
    gameScreen.hidden = true;
    document.body.className = '';
    missingPanel.hidden = false;
  }

  if (!code) {
    showMissing();
    return;
  }

  // --- share link ---

  const shareUrl = `${window.location.origin}/room/${code}`;
  const canShare = navigator.share && window.matchMedia('(pointer: coarse)').matches;
  shareBtn.textContent = `${window.location.host}/room/${code}`;
  shareHint.textContent = canShare ? 'tap to share' : 'tap to copy';

  function copyText(text) {
    if (navigator.clipboard && window.isSecureContext) {
      return navigator.clipboard.writeText(text);
    }
    // Fallback for plain http
    const ta = document.createElement('textarea');
    ta.value = text;
    document.body.appendChild(ta);
    ta.select();
    const ok = document.execCommand('copy');
    ta.remove();
    return ok ? Promise.resolve() : Promise.reject();
  }

  shareBtn.addEventListener('click', () => {
    if (canShare) {
      navigator.share({ title: 'The Court', text: 'Play Go with me', url: shareUrl }).catch(() => {});
      return;
    }
    copyText(shareUrl)
      .then(() => { shareHint.textContent = 'copied'; })
      .catch(() => { shareHint.textContent = 'copy the link above'; });
  });

  // --- rules / board ---

  function applyMove(m) {
    if (m.pass) {
      goEngine.pass();
      boardRenderer.lastMove = null;
    } else {
      const r = goEngine.playMove(m.x, m.y);
      if (r.valid) {
        boardRenderer.lastMove = { x: m.x, y: m.y };
      } else {
        // Both clients run the same rules, so they agree on skipping it;
        // just keep turn order in step with the server
        console.warn('illegal move from server, skipped', m, r.reason);
        goEngine.currentPlayer = goEngine.currentPlayer === 1 ? 2 : 1;
      }
    }
    applied++;
  }

  function replay(moves) {
    goEngine.init(size);
    boardRenderer.lastMove = null;
    applied = 0;
    moves.forEach(applyMove);
  }

  function myTurn() {
    return !result && colorNum[you] === goEngine.currentPlayer;
  }

  function clearPreview() {
    previewCoords = null;
    submitTop.style.display = 'none';
    submitBottom.style.display = 'none';
    if (boardReady) boardRenderer.clearPreview();
  }

  function send(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) ws.send(JSON.stringify(msg));
  }

  // --- render ---

  function statusText() {
    if (result) {
      if (result.reason === 'resign') {
        const loser = result.winner === 'black' ? 'White' : 'Black';
        const winner = result.winner === 'black' ? 'Black' : 'White';
        return `${loser} resigns. ${winner} wins.`;
      }
      return 'Both passed. Count the territory together.';
    }
    if (!ws || ws.readyState !== WebSocket.OPEN) return 'reconnecting…';
    const toPlay = goEngine.currentPlayer === 1 ? 'black' : 'white';
    if (you === 'spectator') return `watching · ${toPlay} to play`;
    const opponent = you === 'black' ? 'white' : 'black';
    let text = myTurn() ? 'your move' : 'their move';
    if (!connected[opponent]) text += ' · your friend has stepped away';
    return text;
  }

  function render() {
    if (missing || !size) return;

    const waiting = you === 'black' && !seated.white && !result;
    waitPanel.hidden = !waiting;
    gameScreen.hidden = waiting;
    document.body.className = waiting ? '' : 'theme-local';
    if (waiting) return;

    // Board must be visible before init so it can measure its container
    if (!boardReady) {
      boardRenderer.init('board', size);
      boardReady = true;
    }
    boardRenderer.board = goEngine.board.map(row => [...row]);
    boardRenderer.draw();

    statusEl.textContent = statusText();
    controls.hidden = you === 'spectator' || !!result;
    passBtn.disabled = !myTurn();
    passBtn.style.visibility = myTurn() ? 'visible' : 'hidden';
  }

  // --- connection ---

  function handle(msg) {
    switch (msg.type) {
      case 'state':
        you = msg.you;
        size = msg.size;
        result = msg.result;
        seated = msg.seated;
        connected = msg.connected;
        clearPreview();
        replay(msg.moves);
        break;
      case 'move':
        if (msg.n < applied) return;       // already have it
        if (msg.n > applied) {             // missed one: reconnect resyncs
          ws.close();
          return;
        }
        applyMove(msg);
        clearPreview();
        break;
      case 'end':
        result = msg.result;
        clearPreview();
        break;
      case 'presence':
        seated = msg.seated;
        connected = msg.connected;
        break;
      case 'error':
        if (msg.reason === 'not_found') return showMissing();
        clearPreview();
        break;
    }
    render();
  }

  function connect() {
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    ws = new WebSocket(`${proto}//${window.location.host}/room/${code}/ws?cid=${encodeURIComponent(clientId)}`);
    ws.onopen = () => { retries = 0; };
    ws.onmessage = (e) => {
      try {
        handle(JSON.parse(e.data));
      } catch (err) {
        console.warn('bad room message', err);
      }
    };
    ws.onclose = (e) => {
      if (missing) return;
      if (e.code === 4404) return showMissing();
      render();
      const delay = Math.min(1000 * 2 ** retries, 10000);
      retries++;
      setTimeout(connect, delay);
    };
  }

  // Keep proxies/tunnels from dropping a quiet connection
  setInterval(() => send({ type: 'ping' }), 30000);

  // --- input ---

  canvas.addEventListener('click', (e) => {
    if (!boardReady || !myTurn()) return;
    const coords = boardRenderer.coordsFromClick(e);
    if (!coords) return;
    if (!goEngine.tryMove(coords.x, coords.y).valid) return;

    boardRenderer.setPreview(coords.x, coords.y, goEngine.currentPlayer);
    previewCoords = coords;
    submitTop.style.display = 'block';
    submitBottom.style.display = 'block';
  });

  function submitMove() {
    if (!previewCoords || !myTurn()) return;
    send({ type: 'move', x: previewCoords.x, y: previewCoords.y, n: applied });
    clearPreview();
  }

  submitTop.addEventListener('click', submitMove);
  submitBottom.addEventListener('click', submitMove);
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && previewCoords) submitMove();
  });

  passBtn.addEventListener('click', () => {
    if (!myTurn()) return;
    send({ type: 'pass', n: applied });
    clearPreview();
  });

  // Resign needs a second tap within 3s
  resignBtn.addEventListener('click', () => {
    if (resignBtn.classList.contains('confirm')) {
      clearTimeout(resignTimer);
      resignBtn.classList.remove('confirm');
      resignBtn.textContent = 'Resign';
      send({ type: 'resign' });
      return;
    }
    resignBtn.classList.add('confirm');
    resignBtn.textContent = 'Resign?';
    resignTimer = setTimeout(() => {
      resignBtn.classList.remove('confirm');
      resignBtn.textContent = 'Resign';
    }, 3000);
  });

  window.addEventListener('resize', () => {
    if (!boardReady) return;
    boardRenderer.setupCanvasSize();
    boardRenderer.draw();
  });

  connect();
}

document.addEventListener('DOMContentLoaded', () => {
  if (document.getElementById('friend-entry')) initFriendEntry();
  if (document.getElementById('game-screen') && document.getElementById('room-wait')) initRoomPage();
});
