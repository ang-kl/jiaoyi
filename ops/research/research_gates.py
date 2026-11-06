"""Shared offline coverage checks and a descriptive candidate-ranking heuristic."""
import collections
import math
import statistics

DATA_GAP_REASONS = frozenset({
    'missing_first_hour_samples',
    'missing_or_flat_preparation_range',
    'missing_next_fresh_entry',
    'no_next_fresh_fill_quote_before_deadline',
    'missing_decision_quote',
    'no_fresh_quote_in_mark_window',
    'missing_band',
    'missing_session_open',
    'missing_close_quote',
    'missing_predictor',
    'zero_tick_stream',
})
LCB_METHOD = 'normal one-sided 90% IID ranking heuristic; not confidence in an edge'


def data_gap_counts(rows):
    """Count each reason at most once per episode, including non-entry episodes."""
    counts = collections.Counter()
    for row in rows:
        study = row['study']
        reasons = {study.get('status')}
        reasons.update(mark.get('reason') for mark in study.get('marks', ()))
        quality = study.get('quality') or {}
        if quality.get('bid_ticks') == 0 or quality.get('ask_ticks') == 0:
            reasons.add('zero_tick_stream')
        counts.update(reasons & DATA_GAP_REASONS)
    return dict(sorted(counts.items()))


def lcb_net_r(values):
    """No sample-size gate is relaxed; correlated/adaptively selected data are not IID."""
    if not values:
        return None
    if any(not math.isfinite(x) for x in values):
        raise ValueError('non-finite research outcome')
    mean = statistics.mean(values)
    return mean if len(values) == 1 else mean - 1.281551565545 * statistics.stdev(values) / math.sqrt(len(values))
