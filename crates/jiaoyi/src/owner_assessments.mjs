// Browser observation only. No trading command or background worker is controlled here.
export function createLiveUpdater({isActive, canPoll, fetchUpdate, applyUpdate, status,
    setTimer = setTimeout, clearTimer = clearTimeout}) {
  let timer = null;
  let flight = null;
  let generation = 0;
  let wanted = false;
  const clear = () => { if (timer !== null) clearTimer(timer); timer = null; };
  function stop() {
    wanted = false;
    generation += 1;
    clear();
    flight?.controller.abort();
    status(isActive() && !canPoll()
      ? 'Updates paused. No known open-market positions; use Recheck to obtain a fresh snapshot.'
      : 'Updates paused. Activate this page to view open-market positions.');
  }
  async function run() {
    timer = null;
    if (!wanted || !isActive() || !canPoll() || flight) return;
    const current = {generation, controller: new AbortController()};
    flight = current;
    const timeout = setTimer(() => current.controller.abort(), 10000);
    try {
      const data = await fetchUpdate(current.controller.signal);
      if (current.generation !== generation || !wanted) return;
      if (!isActive() || !canPoll()) { stop(); return; }
      if (current.controller.signal.aborted) throw new Error('Request timed out');
      const updated = applyUpdate(data);
      status(canPoll() ? updated === 0 ? 'Waiting for fresh evidence; rows retain their displayed snapshot times.' : 'Live open-position rows update every five seconds while this page is active.'
        : 'Updates paused. No known open-market positions; use Recheck to obtain a fresh snapshot.');
    } catch (_) {
      if (current.generation === generation && wanted) {
        wanted = false;
        status('Updates stopped: fresh data unavailable. Use Recheck to try again.');
      }
    } finally {
      clearTimer(timeout);
      if (flight === current) flight = null;
      if (wanted && isActive() && canPoll() && timer === null) {
        if (current.generation === generation) timer = setTimer(run, 5000);
        else void run();
      }
    }
  }
  function sync() {
    if (!isActive() || !canPoll()) { stop(); return; }
    wanted = true;
    if (!flight && timer === null) void run();
  }
  return {sync, stop};
}

export function startOwnerPage(doc, win) {
  const label = doc.getElementById('live-update-status');
  if (!label) return;
  const rows = () => [...doc.querySelectorAll('tr[data-position-key]')];
  const isActive = () => doc.visibilityState === 'visible' && doc.hasFocus();
  const canPoll = () => rows().some(row => Date.parse(row.dataset.marketOpenUntil) > Date.now());
  const updater = createLiveUpdater({
    isActive, canPoll,
    status: message => { label.textContent = message; },
    fetchUpdate: async signal => {
      const reply = await win.fetch('/owner/live-positions', {
        signal, credentials: 'same-origin', cache: 'no-store', headers: {'Accept': 'application/json'}
      });
      if (!reply.ok) throw new Error('Owner read unavailable');
      return reply.json();
    },
    applyUpdate: data => {
      if (!Array.isArray(data.accounts)) throw new Error('Invalid position response');
      const current = new Map(rows().map(row => [row.dataset.positionKey, row]));
      let updated = 0;
      for (const account of data.accounts) {
        const prefix = String(account.account_id) + ':';
        const present = new Set((account.position_ids || []).map(id => prefix + id));
        if (account.evidence_complete === true) {
          for (const [key, row] of current) {
            if (key.startsWith(prefix) && Date.parse(row.dataset.marketOpenUntil) > Date.now() && !present.has(key)) {
              row.remove(); current.delete(key);
            }
          }
        }
        for (const item of account.positions || []) {
          const key = prefix + item.id;
          const old = current.get(key);
          const eligibility = item.live_update || {};
          // Keep a closed-market snapshot unchanged; update its polling gate only.
          if (old) {
            old.dataset.marketOpenUntil = eligibility.market_open_until || '';
            old.dataset.liveEligible = String(eligibility.eligible === true);
            const note = old.querySelector('[data-update-status]');
            if (note) note.textContent = 'Snapshot — ' + String(eligibility.reason || 'freshness unavailable').replaceAll('_', ' ');
          }
          if (!eligibility.eligible || !(Date.parse(eligibility.valid_until) > Date.now())
              || !(Date.parse(eligibility.market_open_until) > Date.now()) || !item.html) continue;
          const body = doc.createElement('tbody');
          body.innerHTML = item.html; // Same-origin server fragment; all broker strings are HTML-escaped.
          const fresh = body.firstElementChild;
          if (fresh?.tagName !== 'TR' || fresh.dataset.positionKey !== key) throw new Error('Invalid row');
          if (old) old.replaceWith(fresh);
          else doc.getElementById('positions-' + account.account_id)?.append(fresh);
          current.set(key, fresh);
          const note = fresh.querySelector('[data-update-status]');
          if (note) note.textContent = 'Updated';
          updated += 1;
        }
      }
      return updated;
    },
    setTimer: (fn, ms) => win.setTimeout(fn, ms), clearTimer: id => win.clearTimeout(id)
  });
  doc.addEventListener('visibilitychange', updater.sync);
  win.addEventListener('focus', updater.sync);
  win.addEventListener('blur', updater.stop);
  win.addEventListener('pagehide', updater.stop);
  win.addEventListener('pageshow', updater.sync);
  updater.sync();
  return updater;
}

if (typeof window !== 'undefined' && typeof document !== 'undefined') startOwnerPage(document, window);
