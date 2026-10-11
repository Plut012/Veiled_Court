// Summon flames flanking the title — the GPU pod, as two flames. No words.
//   bare wick   the court sleeps; nothing is being paid for. Touch to summon.
//   flickering  the spirit is waking (the pod is booting).
//   burning     the spirit is here (the pod is ready, and billing).
//   smoke       touched twice: the flame is out, the pod is still stopping.
// Both flames share one state; either can be touched. Begin still works on its
// own (loading.html summons if needed). No wick without a coordinator (run-local.sh).

(() => {
  const WORD_KEY = 'courtWord';      // shared with loading.html / presence.js
  const QUICK_MS = 4000;             // while waking or smoking
  const WATCH_MS = 20000;            // while burning — notices when the pod rests on its own
  const IDLE_MS = 60000;             // while unlit — notices a summon from another tab
  const GIVE_UP_MS = 10 * 60 * 1000; // waking this long: the coordinator stops the pod
  const SMOKE_MS = 90000;            // smoking this long: RunPod didn't stop it
  const DOUBLE_MS = 450;

  const root = document.getElementById('summon');
  const flames = root.querySelectorAll('.summon-flame');
  const form = document.getElementById('summon-word');
  const input = form.querySelector('input');

  let state = 'asleep';   // asleep | word | waking | lit | smoking
  let wordFor = 'summon'; // what the word form will do: 'summon' | 'rest'
  let since = 0;          // when waking / smoking began
  let timer = null;
  let lastTap = 0;
  let busy = false;       // a summon or rest request is in flight

  function storedWord() {
    try { return localStorage.getItem(WORD_KEY) || ''; } catch (e) { return ''; }
  }
  function rememberWord(w) {
    try { w ? localStorage.setItem(WORD_KEY, w) : localStorage.removeItem(WORD_KEY); } catch (e) {}
  }

  function setState(next) {
    if (next !== state && (next === 'waking' || next === 'smoking')) since = Date.now();
    state = next;
    root.dataset.state = next;
    form.hidden = next !== 'word';
    const label = next === 'lit' || next === 'waking' ? 'the spirit is here — touch twice to let it rest' : 'summon the spirit';
    flames.forEach(f => f.setAttribute('aria-label', label));
    if (next === 'word') input.focus();
  }

  function schedule(ms) {
    clearTimeout(timer);
    timer = setTimeout(check, ms);
  }

  // GET /status → reflect the pod. Returns false when there is no coordinator.
  async function check() {
    let data;
    try {
      const resp = await fetch('/status', { cache: 'no-store' });
      if (resp.status === 404 || resp.status === 405) return false;
      if (!resp.ok) { schedule(QUICK_MS); return true; }  // transient — look again
      data = await resp.json();
    } catch (e) {
      schedule(QUICK_MS);
      return true;
    }
    if (state === 'word' || busy) { schedule(QUICK_MS); return true; }  // don't interrupt

    const pod = data.ready ? 'ready' : (data.status === 'starting' ? 'starting' : 'off');

    if (state === 'smoking') {
      if (pod === 'off') {
        setState('asleep');
        schedule(IDLE_MS);
      } else if (Date.now() - since > SMOKE_MS) {
        setState(pod === 'ready' ? 'lit' : 'waking');  // it didn't go out — touch twice again
        schedule(QUICK_MS);
      } else {
        schedule(QUICK_MS);
      }
      return true;
    }

    if (pod === 'ready') {
      if (state !== 'lit') setState('lit');
      schedule(WATCH_MS);
    } else if (pod === 'starting') {
      if (state !== 'waking') setState('waking');
      if (Date.now() - since > GIVE_UP_MS) {
        setState('asleep');
        schedule(IDLE_MS);
      } else {
        schedule(QUICK_MS);
      }
    } else {
      if (state !== 'asleep') setState('asleep');
      schedule(IDLE_MS);
    }
    return true;
  }

  // POST /summon or /rest → 'ok' | 'wrong' | 'busy' | 'error'
  async function ask(path, password) {
    try {
      const resp = await fetch(path, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password }),
      });
      if (resp.status === 401) return 'wrong';
      if (resp.status === 503) return 'busy';
      return resp.ok ? 'ok' : 'error';
    } catch (e) {
      return 'error';
    }
  }

  async function kindle(word) {
    busy = true;
    setState('waking');
    const result = await ask('/summon', word);
    busy = false;
    if (result === 'wrong') return askWord('summon', true);
    if (result !== 'ok') return setState('asleep');  // the flame doesn't catch
    rememberWord(word);
    window.dispatchEvent(new Event('court-summon'));
    schedule(1000);
  }

  async function snuff(word) {
    const before = state;
    busy = true;
    setState('smoking');
    const result = await ask('/rest', word);
    busy = false;
    if (result === 'wrong') return askWord('rest', true);
    if (result !== 'ok') {
      setState(before);  // it didn't go out
      schedule(QUICK_MS);
      return;
    }
    rememberWord(word);
    window.dispatchEvent(new Event('court-rest'));
    schedule(1500);
  }

  function askWord(purpose, wrong) {
    wordFor = purpose;
    if (wrong) rememberWord('');
    setState('word');
    input.value = '';
    if (wrong) {
      form.classList.remove('shiver');
      void form.offsetWidth; // restart the animation
      form.classList.add('shiver');
    }
  }

  // One touch on an unlit candle summons; two touches on a flame put it out.
  const touch = () => {
    if (busy) return;
    const now = Date.now();
    const burning = state === 'lit' || state === 'waking';

    if (!burning) {
      lastTap = 0;
      if (state === 'smoking') return;
      if (state === 'word') return input.focus();
      const w = storedWord();
      return w ? kindle(w) : askWord('summon');
    }

    if (now - lastTap < DOUBLE_MS) {
      lastTap = 0;
      const w = storedWord();
      return w ? snuff(w) : askWord('rest');
    }
    lastTap = now;
  };
  flames.forEach(f => f.addEventListener('click', touch));

  form.addEventListener('submit', (e) => {
    e.preventDefault();
    const w = input.value.trim();
    if (!w || busy) return;
    input.value = '';
    input.blur();
    wordFor === 'rest' ? snuff(w) : kindle(w);
  });

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') { setState('asleep'); check(); }
  });

  // Elsewhere (presence ember on another page / loading.html) changed the pod — look again
  window.addEventListener('court-rest', () => { if (state !== 'smoking') schedule(2000); });

  // Only light the wicks when a coordinator answers /status
  setState('asleep');
  check().then(found => { if (found) root.classList.add('present'); });
})();
