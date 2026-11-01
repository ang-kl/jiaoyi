import test from 'node:test';
import assert from 'node:assert/strict';
import { createLiveUpdater, priceDirection } from '../../src/owner_assessments.mjs';

const flush = async () => {
  for (let i = 0; i < 12; i += 1) await Promise.resolve();
};

function clock() {
  let at = 0;
  let serial = 0;
  const timers = new Map();
  return {
    now: () => at,
    set: (callback, delay) => {
      const id = ++serial;
      timers.set(id, { callback, due: at + delay });
      return id;
    },
    clear: id => timers.delete(id),
    count: () => timers.size,
    async advance(milliseconds) {
      const target = at + milliseconds;
      let steps = 0;
      while (true) {
        const next = [...timers.entries()]
          .filter(([, item]) => item.due <= target)
          .sort((a, b) => a[1].due - b[1].due || a[0] - b[0])[0];
        if (!next) break;
        assert.ok(++steps < 1000, 'a bounded clock must not hide a busy loop');
        timers.delete(next[0]);
        at = next[1].due;
        next[1].callback();
        await flush();
      }
      at = target;
      await flush();
    },
  };
}

function harness({ active = true, market = true } = {}) {
  const timer = clock();
  const state = { active, market };
  const requests = [];
  const applied = [];
  const statuses = [];
  const pauses = [];
  const controller = createLiveUpdater({
    isActive: () => state.active,
    canPoll: () => state.market,
    fetchUpdate: signal => {
      let resolve;
      let reject;
      const pending = new Promise((yes, no) => { resolve = yes; reject = no; });
      // Deliberately ignore abort until the test settles the promise. This
      // catches stale repaint and concurrent physical request races.
      requests.push({ signal, resolve, reject });
      return pending;
    },
    applyUpdate: value => applied.push(value),
    status: (...values) => statuses.push(values),
    onPause: () => pauses.push(timer.now()),
    setTimer: timer.set,
    clearTimer: timer.clear,
    now: timer.now,
  });
  return { timer, state, requests, applied, statuses, pauses, controller };
}

test('inactive pages and closed/unknown markets make no automatic requests', async () => {
  for (const options of [{ active: false }, { market: false }]) {
    const h = harness(options);
    h.controller.sync();
    await h.timer.advance(30_000);
    assert.equal(h.requests.length, 0);
    assert.equal(h.applied.length, 0);
    assert.equal(h.timer.count(), 0);
    h.controller.stop();
  }
});

test('active open-market updates are single-flight and wait five seconds after completion', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  assert.equal(h.requests.length, 1);
  await h.timer.advance(9_000);
  assert.equal(h.requests.length, 1, 'a slow response must not overlap a second request');
  h.requests[0].resolve({ sequence: 1 });
  await flush();
  assert.deepEqual(h.applied, [{ sequence: 1 }]);
  await h.timer.advance(4_999);
  assert.equal(h.requests.length, 1);
  await h.timer.advance(1);
  assert.equal(h.requests.length, 2);
  h.controller.stop();
});

test('blur aborts the request and a late successful response cannot repaint or restart polling', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  h.state.active = false;
  h.controller.sync();
  assert.equal(h.requests[0].signal.aborted, true);
  h.requests[0].resolve({ stale: true });
  await flush();
  await h.timer.advance(30_000);
  assert.deepEqual(h.applied, []);
  assert.equal(h.requests.length, 1);
  assert.equal(h.timer.count(), 0);
  h.controller.stop();
});

test('rapid resume events wait for the aborted physical request and then resume exactly once', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  h.state.active = false;
  h.controller.sync();
  h.state.active = true;
  h.controller.sync();
  h.controller.sync();
  h.controller.sync();
  await flush();
  assert.equal(h.requests.length, 1);
  h.requests[0].resolve({ stale: true });
  await flush();
  assert.equal(h.requests.length, 2);
  assert.deepEqual(h.applied, []);
  assert.equal(h.requests[1].signal.aborted, false);
  h.controller.sync();
  h.controller.sync();
  assert.equal(h.requests[1].signal.aborted, false, 'duplicate active events do not cancel the current request');
  h.requests[1].resolve({ sequence: 2 });
  await flush();
  assert.deepEqual(h.applied, [{ sequence: 2 }]);
  await h.timer.advance(5_000);
  assert.equal(h.requests.length, 3);
  h.controller.stop();
});

test('a response crossing a market boundary cannot repaint, and no closed-market timer is retained', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  h.state.market = false;
  h.requests[0].resolve({ afterClose: true });
  await flush();
  await h.timer.advance(30_000);
  assert.deepEqual(h.applied, []);
  assert.equal(h.requests.length, 1);
  assert.equal(h.timer.count(), 0);
  h.state.market = true;
  h.controller.sync();
  await flush();
  assert.equal(h.requests.length, 2, 'an explicit active-market recheck can resume');
  h.controller.stop();
});

test('read failures stop automatic retries until an explicit recheck', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  h.requests[0].reject(new Error('owner read unavailable'));
  await flush();
  await h.timer.advance(30_000);
  assert.equal(h.requests.length, 1);
  assert.deepEqual(h.applied, []);
  assert.equal(h.timer.count(), 0);
  h.controller.sync();
  await flush();
  assert.equal(h.requests.length, 2);
  h.controller.stop();
});

test('the ten-second request timeout aborts and refuses a late response without retrying', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  await h.timer.advance(10_000);
  assert.equal(h.requests[0].signal.aborted, true);
  h.requests[0].resolve({ tooLate: true });
  await flush();
  await h.timer.advance(30_000);
  assert.deepEqual(h.applied, []);
  assert.equal(h.requests.length, 1);
  assert.equal(h.timer.count(), 0);
  h.controller.stop();
});

test('pagehide stop stays stopped after late completion even if the document still reports focus', async () => {
  const h = harness();
  h.controller.sync();
  await flush();
  h.controller.stop();
  h.requests[0].resolve({ stale: true });
  await flush();
  await h.timer.advance(30_000);
  assert.deepEqual(h.applied, []);
  assert.equal(h.requests.length, 1);
  assert.equal(h.timer.count(), 0);
});

test('a market closing between requests clears live presentation without making another request', async () => {
  const h = harness();
  h.controller.sync();
  h.requests[0].resolve({ current: true });
  await flush();
  assert.equal(h.pauses.length, 0);
  h.state.market = false;
  await h.timer.advance(5_000);
  assert.equal(h.requests.length, 1);
  assert.equal(h.pauses.length, 1);
  assert.equal(h.timer.count(), 0);
  assert.match(h.statuses.at(-1)[0], /Updates paused/);
});

test('price direction uses a newer comparable exit-side quote, never entry price or inferred P&L', () => {
  const before = {side: 'buy', bid: '100', ask: '102', quoteAt: '2026-11-02T00:00:00Z'};
  const after = {...before, bid: '101', ask: '101.5', quoteAt: '2026-11-02T00:00:01Z'};
  assert.deepEqual(priceDirection(before, after), {direction: 'up', text: 'Price ↑ +1'});
  assert.deepEqual(priceDirection({...before, side: 'sell'}, {...after, side: 'sell'}),
    {direction: 'down', text: 'Price ↓ −0.5'});
  assert.equal(priceDirection(before, {...after, bid: '100'}).direction, 'unchanged');
  for (const current of [
    {...after, quoteAt: before.quoteAt},
    {...after, quoteAt: '2026-11-01T23:59:59Z'},
    {...after, quoteAt: 'invalid'},
    {...after, side: 'sell'},
    {...after, bid: ''},
    {...after, bid: 'NaN'},
  ]) assert.equal(priceDirection(before, current).direction, 'unknown');
  assert.equal(priceDirection(undefined, after).direction, 'unknown');
  assert.equal(priceDirection({...before, bid: ''}, after).direction, 'unknown');
});
