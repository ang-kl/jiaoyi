"""RFC G1/G3/G4/G5/G9: offline identity, dependency and trial barriers.

No market or broker client is imported. Diagnostics may inventory an ineligible
source; only verify_plan can issue evaluation eligibility. Historical parity is
a distinct, digest-pinned operation and never issues a qualification permit.
"""
from dataclasses import dataclass
import ctypes
import ctypes.util
import datetime as dt
import errno
import gzip
import hashlib
import json
from pathlib import Path
import sys
from zoneinfo import ZoneInfo

from replay_inventory import canonical, sha, ms, permitted
from unified_replay import slots


class Refusal(ValueError):
    pass


def require(condition, reason):
    if not condition:
        raise Refusal(reason)


def offline_process():
    """Linux seccomp is inherited by native children; Python hooks alone are not."""
    def deny(event, _):
        if event in ('socket.connect', 'socket.getaddrinfo', 'socket.__new__'):
            raise Refusal('offline_network_forbidden')
    sys.addaudithook(deny)
    name = ctypes.util.find_library('seccomp')
    require(name is not None, 'offline_seccomp_unavailable')
    lib = ctypes.CDLL(name, use_errno=True)
    lib.seccomp_init.argtypes = [ctypes.c_uint32]
    lib.seccomp_init.restype = ctypes.c_void_p
    lib.seccomp_syscall_resolve_name.argtypes = [ctypes.c_char_p]
    lib.seccomp_syscall_resolve_name.restype = ctypes.c_int
    lib.seccomp_rule_add.argtypes = [ctypes.c_void_p, ctypes.c_uint32, ctypes.c_int, ctypes.c_uint]
    lib.seccomp_load.argtypes = [ctypes.c_void_p]
    lib.seccomp_release.argtypes = [ctypes.c_void_p]
    ctx = lib.seccomp_init(0x7fff0000)  # SCMP_ACT_ALLOW; deny every network ingress/egress syscall.
    require(bool(ctx), 'offline_seccomp_init')
    try:
        for call in (b'socket', b'connect', b'socketcall', b'sendto', b'sendmsg', b'sendmmsg'):
            number = lib.seccomp_syscall_resolve_name(call)
            if number >= 0:
                require(lib.seccomp_rule_add(ctx, 0x50000 | errno.EPERM, number, 0) == 0,
                        'offline_seccomp_rule')
        require(lib.seccomp_load(ctx) == 0, 'offline_seccomp_load')
    finally:
        lib.seccomp_release(ctx)
    return 'python_audit_and_inherited_linux_seccomp'


@dataclass(frozen=True)
class Identity:
    candidate_id: str
    hypothesis_sha256: str
    code_sha256: str
    calendar_sha256: str
    contract_sha256: str
    cost_sha256: str
    spent_registry_sha256: str
    rfc_state: str

    @classmethod
    def parse(cls, value):
        require(isinstance(value, dict) and set(value) == set(cls.__annotations__), 'identity_schema')
        for k, v in value.items():
            require(isinstance(v, str) and bool(v), 'identity_missing:' + k)
            if k.endswith('sha256'):
                require(len(v) == 64 and all(c in '0123456789abcdef' for c in v), 'identity_digest:' + k)
        return cls(**value)


@dataclass(frozen=True)
class Interval:
    symbol: str
    account_id: int
    start_ms: int
    end_ms: int
    role: str

    def validate(self):
        require(self.account_id > 0 and bool(self.symbol), 'interval_identity')
        require(permitted(self.start_ms, self.end_ms), 'protected_or_outside_research_interval')

    def overlaps(self, other):
        return (self.account_id, self.symbol) == (other.account_id, other.symbol) and (
            self.start_ms < other.end_ms and other.start_ms < self.end_ms)


def verify_transition(previous, following, transition, registered_transitions):
    """A self-attested override is not a transition. Registry is part of the code freeze."""
    if previous == following:
        require(transition is None, 'spurious_transition')
        return
    require(transition in registered_transitions, 'unregistered_rfc_transition')
    require(transition['from_identity'] == previous and transition['to_identity'] == following,
            'transition_identity_mismatch')
    require(bool(transition.get('owner_trace')) and bool(transition.get('reviewed_commit')),
            'transition_authority_missing')


def verified_json(path, digest, quarantine=()):
    require(digest not in quarantine, 'protected_receipt_quarantined')
    p = Path(path)
    require(p.is_file() and sha(p) == digest, 'receipt_hash_mismatch')
    # A new caller must declare a bounded source; no remote URI is accepted.
    require(p.stat().st_size <= 32_000_000, 'receipt_size_budget')
    return json.loads(p.read_text(), parse_constant=lambda _: (_ for _ in ()).throw(Refusal('nonfinite_json')))


def verify_bars(rows, required, start, receipt_refs, verified_sources):
    require(required and len(required) == len(set(required)), 'invalid_preparation_slots')
    require(set(required) <= set(rows), 'preparation_not_decoded_complete')
    for t in required:
        require(t + 1800000 <= start and permitted(t, t + 1800000), 'noncausal_or_protected_preparation')
        b = rows[t]
        require(b['ts_ms'] == t and t % 1800000 == 0, 'bar_grid')
        require(all(isinstance(b[k], int) and not isinstance(b[k], bool) for k in
                    ('open_e5', 'close_e5', 'high_e5', 'low_e5', 'volume')), 'bar_numeric_type')
        require(0 < b['low_e5'] <= min(b['open_e5'], b['close_e5']) <=
                max(b['open_e5'], b['close_e5']) <= b['high_e5'] and b['volume'] >= 0, 'bar_ohlc')
        refs = receipt_refs.get(t, ())
        require(bool(refs) and all(p in verified_sources for p in refs), 'bar_without_verified_receipt')


def verify_plan(plan, registry, metadata, bars, bar_refs, verified_sources, *, stage):
    """Return a content-bound proof or refuse the entire plan before any replay.

    stage=preparation checks requests; stage=ticks requires decoded M30 first;
    stage=evaluation additionally requires complete executable paths and tail.
    Acquisition remains separately paused by rebuild_authority.
    """
    require(stage in ('preparation', 'ticks', 'evaluation'), 'preflight_stage')
    Identity.parse(plan['identity'])
    verify_transition(registry['identity'], plan['identity'], plan.get('transition'), registry['transitions'])
    require(plan['identity'] == registry['identity'], 'candidate_requires_new_registry_version')
    jobs = plan['episodes']
    require(jobs and len(jobs) == plan['scheduled_episodes'], 'scheduled_denominator')
    require(len({j['episode_id'] for j in jobs}) == len(jobs), 'duplicate_episode')
    proofs = []
    for j in jobs:
        m = metadata[j['episode_id']]
        require(m['symbol'] == j['symbol'] and m['session'] is not None, 'calendar_identity')
        start = ms(m['session']['open'])
        end = start + registry['horizon_ms']
        require((ms(j['from']), ms(j['to'])) == (start, end), 'native_session_window_mismatch')
        i = Interval(j['symbol'], j['account_id'], start, end + registry['tail_ms'], 'ticks')
        i.validate()
        if stage != 'evaluation':
            if j.get('tick_mode') == 'verified_cache':
                require(verified_sources.get(j.get('source_path')) == j.get('tick_receipt_sha256')
                        and bool(j.get('tick_receipt_sha256')), 'tick_cache_not_verified')
            else:
                require(j.get('tick_mode') == 'unspent', 'tick_request_mode_missing')
                require(not any(i.overlaps(Interval(**v)) for v in registry['spent_intervals']), 'spent_tick_request')
        required = slots(m['prior_45_calendar_days'])
        # Never let a subset of dependencies certify its own completeness.
        require(j['required_m30_slots'] == required, '45day_preparation_not_fully_enumerated')
        segments = j['preparation_segments']
        require(bool(segments), 'preparation_intervals_absent')
        for t in required:
            covering = [s for s in segments if s['from_ms'] <= t and t + 1800000 <= s['to_ms']]
            require(len(covering) == 1, 'preparation_partition_gap_or_overlap')
        for s in segments:
            dep = Interval(j['symbol'], j['account_id'], s['from_ms'], s['to_ms'], 'm30')
            dep.validate()
            require(s['mode'] in ('verified_cache', 'unspent'), 'spent_or_uncertain_preparation')
            if s['mode'] == 'verified_cache':
                require(verified_sources.get(s['source_path']) == s['sha256'], 'cache_not_verified')
            else:
                require(not any(dep.overlaps(Interval(**v)) for v in registry['spent_intervals']),
                        'spent_preparation_request')
                require(stage == 'preparation', 'preparation_must_be_decoded_before_ticks')
        if stage != 'preparation':
            key = (j['account_id'], j['symbol'])
            verify_bars(bars.get(key, {}), required, start, bar_refs.get(key, {}), verified_sources)
        if stage == 'evaluation':
            c = j['path_coverage']
            require(j['protocol_status'] == 'clean_first_read', 'inherited_or_unverified_path')
            require(verified_sources.get(j['source_path']) == j['tick_receipt_sha256'], 'unverified_tick_source')
            require(c['opening_fresh'] and c['closing_tail_fresh'], 'missing_fresh_tick_boundary')
            require(c['from_ms'] <= start and c['through_ms'] >= end, 'holding_path_incomplete')
            require(c['unexpected_gaps'] == 0 and c['ordering_ambiguous'] is False, 'unqualified_path_continuity')
            if plan.get('resolution') == 'continuation_72h':
                require(c.get('continuation_through_ms', 0) >= start + 72 * 3600000,
                        'continuation_end_of_cache')
                require(c.get('continuation_missing_slots') == 0, 'continuation_gap')
        proofs.append({'episode_id': j['episode_id'], 'required_slots': len(required)})
    return {'plan_sha256': canonical(plan), 'registry_sha256': canonical(registry),
            'stage': stage, 'episodes': proofs, 'network_authority': False}


def split_budget_guard(candidate, reservation, inspected, trials):
    require(candidate['candidate_id'] == reservation['candidate_id'], 'split_candidate_mismatch')
    require(len(trials) <= reservation['maximum_trials'], 'trial_budget_exceeded')
    require(len({t['trial_id'] for t in trials}) == len(trials), 'duplicate_trial_id')
    require(all(t['status'] in ('failed', 'passed', 'unresolved', 'running') for t in trials), 'trial_status')
    require(set(t['candidate_id'] for t in trials) <= set(reservation['registered_candidates']), 'unregistered_variant')
    intervals = [Interval(**v) for v in reservation['dependencies']]
    for i in intervals:
        i.validate()
        for observed in inspected:
            old = Interval(**observed)
            # Preparation inspection and label leakage remain different reasons;
            # neither can be silently represented as previously unseen.
            if i.overlaps(old):
                raise Refusal('already_inspected_' + old.role)
    embargo = reservation['embargo_ms']
    require(isinstance(embargo, int) and embargo >= reservation['maximum_holding_ms'], 'embargo_too_short')
    for left in reservation['development_labels']:
        a = Interval(**left)
        a.validate()
        for right in intervals:
            extended = Interval(a.symbol, a.account_id, a.start_ms, a.end_ms + embargo, a.role)
            require(not extended.overlaps(right), 'label_overlap_or_embargo')
    return {'status': 'PASSED', 'candidate_id': candidate['candidate_id'], 'trials': len(trials)}


@dataclass(frozen=True)
class CalendarTerms:
    """Broker/reference/strategy layers never substitute for each other."""
    layer: str
    version: str
    timezone: str
    effective_from: str
    effective_to: str
    authority: str
    source_sha256: str
    exceptions: dict
    maintenance: tuple
    funding_time: str | None

    def local_instant(self, date, time):
        require(self.layer in ('broker', 'reference_venue', 'strategy'), 'calendar_layer')
        require(self.effective_from <= date <= self.effective_to, 'calendar_outside_effective_dates')
        require(self.authority not in ('unknown', ''), 'historical_calendar_unverified')
        value = dt.datetime.fromisoformat(date + 'T' + time)
        zone = ZoneInfo(self.timezone)
        first = value.replace(tzinfo=zone, fold=0)
        second = value.replace(tzinfo=zone, fold=1)
        require(first.utcoffset() == second.utcoffset(), 'ambiguous_or_nonexistent_local_time')
        utc = first.astimezone(dt.timezone.utc)
        require(utc.astimezone(zone).replace(tzinfo=None) == value, 'nonexistent_local_time')
        return utc.isoformat()

    def close(self, date, default):
        override = self.exceptions.get(date)
        if override == 'closed':
            return None
        return self.local_instant(date, override or default)


class PartitionStore:
    """Content-addressed, compressed local partitions; never deletes raw receipts."""
    def __init__(self, root):
        self.root = Path(root)
        self.root.mkdir(parents=True, exist_ok=True)

    def put(self, raw, consumer, expires_at):
        require(bool(consumer) and bool(expires_at), 'partition_consumer_and_retention_required')
        require(len(raw) <= 32_000_000, 'partition_budget')
        digest = hashlib.sha256(raw).hexdigest()
        p = self.root / (digest + '.json.gz')
        if p.exists():
            require(gzip.decompress(p.read_bytes()) == raw, 'immutable_partition_changed')
        else:
            with p.open('xb') as f:
                f.write(gzip.compress(raw, mtime=0))
        return {'sha256': digest, 'compressed_sha256': sha(p), 'path': str(p),
                'consumer': consumer, 'expires_at': expires_at, 'bytes': len(raw), 'stored_bytes': p.stat().st_size}

    def read(self, ref):
        require(ref['consumer'] and ref['expires_at'], 'unregistered_partition')
        p = self.root / (ref['sha256'] + '.json.gz')
        require(p.is_file() and sha(p) == ref['compressed_sha256'], 'partition_hash_mismatch')
        with gzip.open(p, 'rb') as f:
            raw = f.read(32_000_001)
        require(len(raw) <= 32_000_000 and hashlib.sha256(raw).hexdigest() == ref['sha256'], 'partition_content_mismatch')
        return raw
