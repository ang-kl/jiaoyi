// Local navigation of the displayed snapshot. No polling or data requests.
for (const link of document.querySelectorAll('.dashboard-nav a[href^="#"]')) {
  link.addEventListener('click', () => {
    const target = document.getElementById(link.hash.slice(1));
    if (target instanceof HTMLDetailsElement) target.open = true;
  });
}
for (const card of document.querySelectorAll('[data-trigger-pager]')) {
  const rows = [...card.querySelectorAll('tbody tr[data-trigger-row]')];
  if (!rows.length) continue;
  const viewport = card.querySelector('.trigger-scroll');
  const controls = document.createElement('div');
  controls.className = 'trigger-pagination';
  controls.setAttribute('role', 'group');
  controls.setAttribute('aria-label', 'Distance to trigger pages');
  const previous = document.createElement('button');
  previous.type = 'button';
  previous.textContent = 'Previous';
  previous.dataset.pagePrev = '';
  const next = document.createElement('button');
  next.type = 'button';
  next.textContent = 'Next';
  next.dataset.pageNext = '';
  const status = document.createElement('span');
  status.dataset.pageStatus = '';
  status.setAttribute('role', 'status');
  status.setAttribute('aria-live', 'polite');
  controls.append(previous, status, next);
  card.prepend(controls);
  const size = 20;
  const pages = Math.ceil(rows.length / size);
  let page = 0;
  function render() {
    const start = page * size;
    rows.forEach((row, i) => { row.hidden = i < start || i >= start + size; });
    previous.disabled = page === 0;
    next.disabled = page === pages - 1;
    status.textContent = `Rows ${start + 1}–${Math.min(start + size, rows.length)} of ${rows.length} · Page ${page + 1} of ${pages}`;
    viewport.scrollTop = 0;
  }
  previous.addEventListener('click', () => { if (page > 0) { page -= 1; render(); } });
  next.addEventListener('click', () => { if (page + 1 < pages) { page += 1; render(); } });
  render();
}
