// Presence ember — a small light in the corner while the GPU pod runs (and costs).
// No light: the court is empty, nothing is being paid for.
// Flickering: the spirit is waking. Breathing: the spirit is here.
// Tap it to see when the spirit will rest on its own, or to let it rest now.
// Hidden entirely when there is no coordinator (run-local.sh).

(() => {
  const WORD_KEY = 'courtWord';   // shared with loading.html / summon.js
  const WATCH_MS = 20000;
  const QUICK_MS = 4000;          // while waking or resting

  const el = document.createElement('div');
  el.id = 'presence';
  el.hidden = true;
  el.innerHTML = `
    <button id="presence-ember" aria-label="the spirit's presence"></button>
    <div id="presence-veil" hidden>
      <p id="presence-state"></p>
      <p id="presence-time"></p>
      <button id="presence-rest">let it rest</button>
      <form id="presence-word" hidden autocomplete="off">
        <input type="password" placeholder="speak the word" aria-label="the word" enterkeyhint="go">
      </form>
    </div>`;
  document.body.appendChild(el);

  const ember = el.querySelector('#presence-ember');
  const veil = el.querySelector('#presence-veil');
  const stateLine = el.querySelector('#presence-state');
  const timeLine = el.querySelector('#presence-time');
  const restBtn = el.querySelector('#presence-rest');
  const form = el.querySelector('#presence-word');
  const input = form.querySelector('input');

  let status = 'off';
  let restsIn = null;
  let resting = false;      // a rest was requested; waiting for the pod to stop
  let restingSince = 0;
  let confirmTimer = null;
  let timer = null;

  function storedWord() {
    try { return localStorage.getItem(WORD_KEY) || ''; } catch (e) { return ''; }
  }
  function rememberWord(w) {
    try { w ? localStorage.setItem(WORD_KEY, w) : localStorage.removeItem(WORD_KEY); } catch (e) {}
  }

  function render() {
    el.dataset.state = resting ? 'resting' : status;
    // Absent when off — unless the veil is open on a message
    el.hidden = status === 'off' && !resting && veil.hidden;

    if (resting) {
      stateLine.textContent = 'the spirit is leaving…';
      timeLine.textContent = '';
      restBtn.hidden = true;
    } else if (status === 'ready') {
      stateLine.textContent = 'the spirit is here';
      timeLine.textContent = restsIn == null ? ''
        : restsIn < 60 ? 'rests within the minute'
        : `rests on its own in ~${Math.ceil(restsIn / 60)} min`;
      restBtn.hidden = !form.hidden;
    } else if (status === 'starting') {
      stateLine.textContent = 'the spirit is waking';
      timeLine.textContent = '';
      restBtn.hidden = !form.hidden;
    } else {
      stateLine.textContent = 'the court is empty';
      timeLine.textContent = '';
      restBtn.hidden = true;
    }
  }

  function schedule() {
    clearTimeout(timer);
    timer = setTimeout(check, resting || status === 'starting' ? QUICK_MS : WATCH_MS);
  }

  async function check() {
    try {
      const resp = await fetch('/status', { cache: 'no-store' });
      if (resp.status === 404 || resp.status === 405) return;  // no coordinator — stay hidden
      if (!resp.ok) return schedule();                        // transient — look again later
      const data = await resp.json();
      status = data.ready ? 'ready' : (data.status === 'starting' ? 'starting' : 'off');
      restsIn = typeof data.rests_in === 'number' ? data.rests_in : null;
    } catch (e) {
      return schedule();
    }

    if (resting && status === 'off') {
      resting = false;
      stateLine.textContent = 'the spirit rests';
      setTimeout(closeVeil, 2500);
    } else if (resting && Date.now() - restingSince > 90000) {
      resting = false;                 // RunPod didn't stop it — say so, allow retry
      openVeil();
      render();
      stateLine.textContent = 'the spirit lingers — try again';
      return schedule();
    }
    render();
    schedule();
  }

  // ── veil (the small panel under the ember) ──

  function openVeil() {
    veil.hidden = false;
    el.classList.add('open');
  }
  function closeVeil() {
    veil.hidden = true;
    form.hidden = true;
    el.classList.remove('open');
    resetConfirm();
    render();
  }

  ember.addEventListener('click', () => (veil.hidden ? openVeil() : closeVeil()));
  document.addEventListener('click', (e) => { if (!el.contains(e.target) && !veil.hidden) closeVeil(); });
  document.addEventListener('keydown', (e) => { if (e.key === 'Escape' && !veil.hidden) closeVeil(); });

  // ── letting it rest: tap once, then once more to confirm ──

  function resetConfirm() {
    clearTimeout(confirmTimer);
    restBtn.classList.remove('confirm');
    restBtn.textContent = 'let it rest';
  }

  restBtn.addEventListener('click', () => {
    if (!restBtn.classList.contains('confirm')) {
      restBtn.classList.add('confirm');
      restBtn.textContent = 'let it rest? · yes';
      confirmTimer = setTimeout(resetConfirm, 4000);
      return;
    }
    resetConfirm();
    const w = storedWord();
    if (!w) return askForWord();
    rest(w);
  });

  function askForWord() {
    restBtn.hidden = true;
    form.hidden = false;
    input.focus();
  }

  form.addEventListener('submit', (e) => {
    e.preventDefault();
    const w = input.value.trim();
    if (w) rest(w);
  });

  async function rest(word) {
    let resp;
    try {
      resp = await fetch('/rest', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password: word }),
      });
    } catch (e) {
      stateLine.textContent = 'the court is silent — try again';
      return;
    }

    if (resp.status === 401) {
      rememberWord('');
      input.value = '';
      form.classList.remove('shiver');
      void form.offsetWidth; // restart the animation
      form.classList.add('shiver');
      return askForWord();
    }
    if (!resp.ok) {
      stateLine.textContent = 'the spirit lingers — try again';
      return;
    }

    rememberWord(word);
    input.value = '';
    form.hidden = true;
    resting = true;
    restingSince = Date.now();
    render();
    window.dispatchEvent(new Event('court-rest'));
    clearTimeout(timer);
    timer = setTimeout(check, 1500);
  }

  // Other scripts (summon line) can ask for a fresh look
  window.addEventListener('court-summon', () => { clearTimeout(timer); timer = setTimeout(check, 1500); });

  check();
})();
