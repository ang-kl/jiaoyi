# Independent reference implementation of the H1 daily-bar rules, written from
# docs/preregistration-2026-10-24.md (not from the Rust code), in exact
# decimals (fractions) so the hand arithmetic can be printed digit by digit.
import json, sys
from fractions import Fraction as F
from decimal import Decimal

def load(path):
    d = json.load(open(path))
    return [dict(date=b['bar_date'], open_utc=b['bar_open_utc'],
                 o=F(Decimal(b['open'])), h=F(Decimal(b['high'])),
                 l=F(Decimal(b['low'])), c=F(Decimal(b['close']))) for b in d['bars']]

def ema(vals, L):
    out = [None]*len(vals)
    if len(vals) < L: return out
    a = F(2, L+1)
    e = sum(vals[:L]) / L
    out[L-1] = e
    for i in range(L, len(vals)):
        e = e + a*(vals[i]-e)
        out[i] = e
    return out

def atr(bars, L=20):
    n = len(bars); out = [None]*n
    def tr(i):
        b, pc = bars[i], bars[i-1]['c']
        return max(b['h']-b['l'], abs(b['h']-pc), abs(b['l']-pc))
    a = sum(tr(i) for i in range(1, L+1)) / L
    out[L] = a
    for i in range(L+1, n):
        a = (a*(L-1) + tr(i)) / L
        out[i] = a
    return out

def sim(bars, N=50, k=3, cost=None, first_only=True, log=None):
    """cost: dict(to_ask, to_bid, spread, slip_mult, comm, swap_long, swap_short) constants."""
    c = [b['c'] for b in bars]
    ef, es, at = ema(c, 50), ema(c, 100), atr(bars, 20)
    slip = cost['slip_mult']*cost['spread']
    trades = []
    t = max(100, N)
    last = len(bars)-1
    while t <= last:
        if ef[t-1] is None or es[t-1] is None or at[t-1] is None:
            t += 1; continue
        side = None
        if ef[t-1] > es[t-1]:
            lvl = max(b['h'] for b in bars[t-N:t])
            if bars[t]['h'] >= lvl: side = 'long'
        elif ef[t-1] < es[t-1]:
            lvl = min(b['l'] for b in bars[t-N:t])
            if bars[t]['l'] <= lvl: side = 'short'
        if side is None:
            t += 1; continue
        long = side == 'long'
        ref = max(lvl, bars[t]['o']) if long else min(lvl, bars[t]['o'])
        fill = ref + cost['to_ask'] + slip if long else ref - cost['to_bid'] - slip
        D = k*at[t-1]
        stop = fill - D if long else fill + D
        path = [(bars[t]['date'], 'entry', ref, fill, D, stop)]
        exit = None
        b = bars[t]
        if (long and b['l'] <= stop) or ((not long) and b['h'] >= stop):
            exit = (t, stop, 'stop', False)
        best = b['h'] if long else b['l']
        ambiguous = False
        if exit is None:
            stop = max(stop, best - k*at[t]) if long else min(stop, best + k*at[t])
            path.append((b['date'], 'trail', best, at[t], stop))
            for u in range(t+1, last+1):
                b = bars[u]
                if long:
                    gap, hit, newbest = b['o'] <= stop, b['l'] <= stop, b['h'] > best
                else:
                    gap, hit, newbest = b['o'] >= stop, b['h'] >= stop, b['l'] < best
                if gap:
                    exit = (u, b['o'], 'gap', newbest); break
                if hit:
                    exit = (u, stop, 'stop', newbest); break
                best = max(best, b['h']) if long else min(best, b['l'])
                stop = max(stop, best - k*at[u]) if long else min(stop, best + k*at[u])
                path.append((b['date'], 'trail', best, at[u], stop))
        if exit is None:
            exit = (last, bars[last]['c'], 'period_end', False)
        tx, xref, why, amb = exit
        xfill = xref - cost['to_bid'] - slip if long else xref + cost['to_ask'] + slip
        gross = (xfill - fill)/D if long else (fill - xfill)/D
        comm = 2*cost['comm']/D
        nights = tx - t
        sw = (cost['swap_long'] if long else cost['swap_short'])*nights/D
        trades.append(dict(side=side, t=t, entry_date=bars[t]['date'], level=lvl, ref=ref, fill=fill,
                           atr=at[t-1], D=D, stop0=path[0][5], exit_t=tx, exit_date=bars[tx]['date'],
                           xref=xref, xfill=xfill, why=why, ambiguous=amb, nights=nights,
                           gross=gross, comm=comm, swap=sw, net=gross-comm+sw, path=path,
                           ef=ef[t-1], es=es[t-1]))
        if first_only: return trades
        t = tx + 1
    return trades
