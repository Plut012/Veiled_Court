// Summon line on the selection screen — wake the court before choosing a spirit.
// Begin still works on its own (loading.html summons if needed); this just lets
// the pod boot while the player browses. Hidden when there is no coordinator.

(() => {
  const WORD_KEY = 'courtWord';      // shared with loading.html
  const POLL_MS = 5000;              // while waking
  const WATCH_MS = 30000;            // while awake — notices when the pod sleeps again
  const GIVE_UP_MS = 10 * 60 * 1000;

  const root = document.getElementById('summon');
  const callBtn = document.getElementById('summon-call');
  const form = document.getElementById('summon-word');
  const input = form.querySelector('input');
  const note = document.getElementById('summon-note');

  let timer = null;
  let wakingSince = 0;

  function storedWord() {
    try { return localStorage.getItem(WORD_KEY) || ''; } catch (e) { return ''; }
  }
  function rememberWord(w) {
    try { w ? localStorage.setItem(WORD_KEY, w) : localStorage.removeItem(WORD_KEY); } catch (e) {}
  }

  // States: asleep | word | waking | awake | notice. The note line carries the text.
  function setState(state, text) {
    root.dataset.state = state;
    note.textContent = text || '';
    callBtn.hidden = state !== 'asleep';
    form.hidden = state !== 'word';
    note.hidden = state === 'asleep' || state === 'word';
    if (state === 'word') input.focus();
  }

  function schedule(ms) {
    clearTimeout(timer);
    timer = setTimeout(check, ms);
  }

  // GET /status → reflect the pod. Returns false when there is no coordinator.
  async function check() {
    let data;
    try {
      const resp = await fetch('/status');
      if (!resp.ok) return false;
      data = await resp.json();
    } catch (e) {
      return false;
    }

    if (data.ready) {
      setState('awake', 'the court is awake');
      schedule(WATCH_MS);
    } else if (data.status === 'starting') {
      if (!wakingSince) wakingSince = Date.now();
      if (Date.now() - wakingSince > GIVE_UP_MS) {
        wakingSince = 0;
        setState('asleep');
        return true;
      }
      setState('waking', 'waking…');
      schedule(POLL_MS);
    } else {
      wakingSince = 0;
      setState('asleep');
    }
    return true;
  }

  // POST /summon → 'ok' | 'wrong' | 'busy' | 'error'
  async function summon(password) {
    try {
      const resp = await fetch('/summon', {
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

  function settle(result, password) {
    if (result === 'ok') {
      rememberWord(password);
      wakingSince = Date.now();
      setState('waking', 'waking…');
      schedule(1000);
    } else {
      setState('notice', result === 'busy'
        ? 'no spirit can be roused just now'
        : 'the court is silent');
      setTimeout(() => setState('asleep'), 4000);
    }
  }

  callBtn.addEventListener('click', async () => {
    const remembered = storedWord();
    if (!remembered) return setState('word');
    const result = await summon(remembered);
    if (result === 'wrong') {
      rememberWord('');
      return setState('word');
    }
    settle(result, remembered);
  });

  let speaking = false;
  form.addEventListener('submit', async (e) => {
    e.preventDefault();
    const w = input.value.trim();
    if (!w || speaking) return;
    speaking = true;
    const result = await summon(w);
    speaking = false;

    if (result === 'wrong') {
      input.value = '';
      form.classList.remove('shiver');
      void form.offsetWidth; // restart the animation
      form.classList.add('shiver');
      input.focus();
      return;
    }
    input.value = '';
    input.blur();
    settle(result, w);
  });

  input.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') setState('asleep');
  });

  // Only show the line when a coordinator answers /status
  check().then(found => { if (found) root.hidden = false; });
})();
