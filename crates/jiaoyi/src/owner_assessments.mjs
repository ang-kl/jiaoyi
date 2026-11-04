// Browser observation only. No trading command or background worker is controlled here.
export function createLiveUpdater({isActive, canPoll, fetchUpdate, applyUpdate, status, onPause = () => {},
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
    onPause();
    status(isActive() && !canPoll()
      ? 'Updates paused. No known open-market positions; use Recheck to obtain a fresh snapshot.'
      : 'Updates paused. Activate this page to view open-market positions.');
  }
  async function run() {
    timer = null;
    if (!wanted || flight) return;
    if (!isActive() || !canPoll()) { stop(); return; }
    const current = {generation, controller: new AbortController()};
    flight = current;
    const timeout = setTimer(() => current.controller.abort(), 10000);
    try {
      const data = await fetchUpdate(current.controller.signal);
      if (current.generation !== generation || !wanted) return;
      if (!isActive() || !canPoll()) { stop(); return; }
      if (current.controller.signal.aborted) throw new Error('Request timed out');
      const updated = applyUpdate(data);
      if (!canPoll()) { stop(); return; }
      status(updated === 0 ? 'Waiting for fresh evidence; positions retain their displayed snapshot times.' : 'Live open positions update every five seconds while this page is active.');
    } catch (_) {
      if (current.generation === generation && wanted) {
        wanted = false;
        onPause();
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

// Movement of the executable exit price, not profit/loss or a signal. An ask
// move is relevant to a short; a bid move is relevant to a long.
export function priceDirection(previous, current) {
  const unknown = {direction: 'unknown', text: 'Awaiting comparable quote'};
  if (!previous || previous.side !== current.side) return unknown;
  const field = current.side === 'buy' ? 'bid' : current.side === 'sell' ? 'ask' : null;
  const earlier = Date.parse(previous.quoteAt);
  const later = Date.parse(current.quoteAt);
  if (!field || !Number.isFinite(earlier) || !Number.isFinite(later) || later <= earlier) return unknown;
  if (!String(previous[field] ?? '').trim() || !String(current[field] ?? '').trim()) return unknown;
  const before = Number(previous[field]);
  const after = Number(current[field]);
  if (!Number.isFinite(before) || !Number.isFinite(after)) return unknown;
  const delta = after - before;
  if (delta === 0) return {direction: 'unchanged', text: 'Price unchanged'};
  const amount = new Intl.NumberFormat('en-GB', {maximumSignificantDigits: 8}).format(Math.abs(delta));
  return delta > 0
    ? {direction: 'up', text: `Price ↑ +${amount}`}
    : {direction: 'down', text: `Price ↓ −${amount}`};
}

export function startOwnerPage(doc, win) {
  const label = doc.getElementById('live-update-status');
  if (!label) return;
  const rows = () => [...doc.querySelectorAll('[data-position-key]')];
  const expanded = element => {
    for (let section = element.closest('details'); section; section = section.parentElement?.closest('details')) {
      if (!section.open) return false;
    }
    return true;
  };
  const isActive = () => doc.visibilityState === 'visible' && doc.hasFocus() && expanded(label);
  // A known-open window ending (including the UTC-midnight lease boundary)
  // permits at most two metadata reads, five seconds apart. The first may race
  // the observer's one-second publication. Unknown/closed windows never poll.
  const checkedWindows = new Map();
  const canPoll = () => rows().some(row => {
    const window = row.dataset.marketOpenUntil;
    const until = Date.parse(window);
    const checked = checkedWindows.get(row.dataset.positionKey);
    return Number.isFinite(until) && (until > Date.now() || checked?.window !== window || checked.attempts < 2);
  });
  let expiryTimer = null;
  const clearExpiry = () => { if (expiryTimer !== null) win.clearTimeout(expiryTimer); expiryTimer = null; };
  function pauseRow(row, message) {
    row.classList.remove('position-live');
    const direction = row.querySelector('[data-price-direction]');
    if (direction) {
      direction.dataset.direction = 'unknown';
      direction.textContent = 'Awaiting comparable quote';
    }
    const note = row.querySelector('[data-update-status]');
    if (note && message) note.textContent = message;
  }
  function expireIndicators() {
    clearExpiry();
    const now = Date.now();
    let next = Infinity;
    let expired = false;
    for (const row of rows()) {
      if (!row.classList.contains('position-live')) continue;
      const until = Date.parse(row.dataset.liveUntil);
      if (!isActive() || !Number.isFinite(until) || until <= now) {
        pauseRow(row, 'Snapshot — fresh evidence expired');
        expired = true;
      }
      else next = Math.min(next, until);
    }
    if (Number.isFinite(next)) expiryTimer = win.setTimeout(expireIndicators, Math.max(1, next - now));
    else if (expired) label.textContent = 'Waiting for fresh evidence; positions retain their displayed snapshot times.';
  }
  const updater = createLiveUpdater({
    isActive, canPoll,
    status: message => { label.textContent = message; },
    onPause: () => {
      clearExpiry();
      for (const row of rows()) pauseRow(row, row.classList.contains('position-live') ? 'Snapshot — updates paused' : null);
    },
    fetchUpdate: async signal => {
      const reply = await win.fetch('/owner/live-positions', {
        signal, credentials: 'same-origin', cache: 'no-store', headers: {'Accept': 'application/json'}
      });
      if (!reply.ok) throw new Error('Owner read unavailable');
      return reply.json();
    },
    applyUpdate: data => {
      const current = new Map(rows().map(row => [row.dataset.positionKey, row]));
      for (const key of checkedWindows.keys()) if (!current.has(key)) checkedWindows.delete(key);
      // Count expired-window reads before applying new metadata. A second read
      // can cross the observer publication boundary; only a verified future
      // window can resume repainting. Closed/unknown replies stop immediately.
      for (const [key, row] of current) {
        const window = row.dataset.marketOpenUntil;
        const until = Date.parse(window);
        if (Number.isFinite(until) && until <= Date.now()) {
          const previous = checkedWindows.get(key);
          checkedWindows.set(key, {window, attempts: previous?.window === window ? previous.attempts + 1 : 1});
        }
      }
      if (!Array.isArray(data.accounts)) throw new Error('Invalid position response');
      let updated = 0;
      for (const account of data.accounts) {
        const prefix = String(account.account_id) + ':';
        const complete = account.evidence_complete === true && Array.isArray(account.position_ids);
        const present = new Set((Array.isArray(account.position_ids) ? account.position_ids : []).map(id => prefix + id));
        const accountRows = doc.getElementById('positions-' + account.account_id);
        if (complete) {
          for (const [key, row] of current) {
            // A complete broker book can confirm closure even when the old
            // display lease expired; removing a closed trade is not a quote update.
            if (key.startsWith(prefix) && !present.has(key)) {
              row.remove(); current.delete(key); checkedWindows.delete(key);
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
            pauseRow(old, 'Snapshot — ' + String(eligibility.reason || 'freshness unavailable').replaceAll('_', ' '));
          }
          const cards = accountRows?.dataset.positionLayout === 'cards';
          const fragment = cards ? item.card_html : item.html;
          if (!complete || !eligibility.eligible || !(Date.parse(eligibility.valid_until) > Date.now())
              || !(Date.parse(eligibility.market_open_until) > Date.now()) || !fragment || !accountRows) continue;
          const body = doc.createElement(cards ? 'template' : 'tbody');
          body.innerHTML = fragment; // Same-origin server fragment; all broker strings are HTML-escaped.
          const fresh = cards ? body.content.firstElementChild : body.firstElementChild;
          if (fresh?.tagName !== (cards ? 'ARTICLE' : 'TR') || fresh.dataset.positionKey !== key) throw new Error('Invalid position');
          const previousChecks = old?.querySelector('.position-checks');
          const nextChecks = fresh.querySelector('.position-checks');
          if (previousChecks && nextChecks) nextChecks.open = previousChecks.open;
          const movement = priceDirection(old?.dataset, fresh.dataset);
          const direction = fresh.querySelector('[data-price-direction]');
          if (direction) {
            direction.dataset.direction = movement.direction;
            direction.textContent = movement.text;
          }
          fresh.dataset.liveUntil = eligibility.valid_until;
          fresh.classList.add('position-live');
          if (old) old.replaceWith(fresh);
          else {
            accountRows?.querySelector('[data-empty-position]')?.remove();
            accountRows.append(fresh);
          }
          current.set(key, fresh);
          const note = fresh.querySelector('[data-update-status]');
          if (note) note.textContent = 'Updated';
          updated += 1;
        }
        if (complete && account.position_ids.length === 0 && accountRows?.dataset.positionLayout === 'cards'
            && !accountRows.querySelector('[data-position-key]') && !accountRows.querySelector('[data-empty-position]')) {
          const empty = doc.createElement('p');
          empty.dataset.emptyPosition = '';
          empty.className = 'empty';
          empty.textContent = 'No open positions in the complete broker snapshot.';
          accountRows.append(empty);
        }
      }
      expireIndicators();
      return updated;
    },
    setTimer: (fn, ms) => win.setTimeout(fn, ms), clearTimer: id => win.clearTimeout(id)
  });
  doc.addEventListener('visibilitychange', updater.sync);
  doc.addEventListener('toggle', event => { if (event.target.contains?.(label)) updater.sync(); }, true);
  win.addEventListener('focus', updater.sync);
  win.addEventListener('blur', updater.stop);
  win.addEventListener('pagehide', updater.stop);
  win.addEventListener('pageshow', updater.sync);
  updater.sync();
  return updater;
}

if (typeof window !== 'undefined' && typeof document !== 'undefined') startOwnerPage(document, window);
