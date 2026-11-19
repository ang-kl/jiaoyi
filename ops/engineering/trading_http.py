"""Explicit Trading transport for a staged private-access migration.

No sockets are opened at import/configuration time. The legacy mode preserves
the currently deployed server contract; scoped mode never falls back to legacy
credentials. This module changes client transport only, not broker authority.
"""
import os
import re
import urllib.parse
import urllib.request

DEFAULT_ORIGIN = "https://jiaoyi.up.railway.app"
AUDIT_PATHS = frozenset({
    "/health", "/phase1/report", "/research/inventory", "/research/bars",
    "/research/jobs", "/research/trials", "/trading/costs", "/trading/orders",
    "/trading/executions", "/operations/reports", "/operations/stock-preparation",
    "/operations/stop-loss-reviews", "/operations/trailing-receipts",
    "/operations/setup-assessments", "/owner/setup-assessments",
    "/owner/live-positions", "/owner/quote-coverage",
})
RESEARCH_PATHS = frozenset({"/owner/backtests", "/owner/market-data"})


class NoRedirect(urllib.request.HTTPRedirectHandler):
    """Never forward a credential or replay a research POST after a redirect."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def _env(env):
    return os.environ if env is None else env


def access_mode(env=None):
    mode = _env(env).get("JIAOYI_TRADING_ACCESS_MODE", "legacy")
    if mode not in ("legacy", "scoped"):
        raise ValueError("invalid_trading_access_mode")
    return mode


def _parse_origin(value):
    if (not isinstance(value, str) or not value or len(value) > 2048
            or any(c.isspace() or ord(c) < 32 or ord(c) == 127 for c in value)
            or "?" in value or "#" in value or "\\" in value):
        raise ValueError("invalid_trading_origin")
    try:
        parsed = urllib.parse.urlsplit(value)
        host, port = parsed.hostname, parsed.port
        if (parsed.scheme not in ("http", "https") or not host
                or parsed.username is not None or parsed.password is not None
                or parsed.path not in ("", "/") or "%" in parsed.netloc
                or not host.isascii() or port == 0):
            raise ValueError
        # Keep only a normalized origin. No credentials, path or query may be
        # smuggled into either request URLs or evidence receipts.
        authority = "[" + host.lower() + "]" if ":" in host else host.lower()
        if port is not None:
            authority += ":" + str(port)
        return parsed.scheme + "://" + authority, host.lower(), port
    except (ValueError, UnicodeError):
        raise ValueError("invalid_trading_origin") from None


def origin(base=None, env=None):
    env = _env(env)
    if base is None:
        primary, alias = env.get("TRADING_BASE_URL"), env.get("JIAOYI_BASE_URL")
        if primary is not None and alias is not None:
            if _parse_origin(primary)[0] != _parse_origin(alias)[0]:
                raise ValueError("conflicting_trading_origins")
        base = primary if primary is not None else alias if alias is not None else DEFAULT_ORIGIN
    normalized, host, port = _parse_origin(base)
    if normalized.startswith("http:"):
        # Plain HTTP is never enabled globally. The exact approved private
        # destination must be pinned separately, including its explicit port.
        pin = env.get("JIAOYI_TRADING_PRIVATE_ORIGIN")
        if not pin or _parse_origin(pin)[0] != normalized:
            raise ValueError("private_origin_not_pinned")
        if (port is None or not host.endswith(".railway.internal")
                or not re.fullmatch(r"[a-z0-9](?:[a-z0-9.-]*[a-z0-9])?", host)
                or ".." in host):
            raise ValueError("invalid_private_trading_origin")
    return normalized


def _token(value):
    if (not isinstance(value, str) or not value or len(value) > 4096
            or not value.isascii() or any(ord(c) < 32 or ord(c) == 127 for c in value)):
        raise ValueError("trading_credential_missing_or_invalid")
    return value


def research_token(env=None):
    env = _env(env)
    key = "JIAOYI_RESEARCH_BEARER_TOKEN" if access_mode(env) == "scoped" else "JIAOYI_UI_PASSWORD"
    return _token(env.get(key))


def request(route, *, purpose="audit", base=None, env=None, data=None, token=None):
    env = _env(env)
    mode = access_mode(env)
    if (not isinstance(route, str) or not route.startswith("/") or route.startswith("//")
            or "#" in route or "\\" in route
            or any(c.isspace() or ord(c) < 32 or ord(c) == 127 for c in route)):
        raise ValueError("invalid_trading_route")
    parsed = urllib.parse.urlsplit(route)
    if parsed.scheme or parsed.netloc:
        raise ValueError("invalid_trading_route")
    path = parsed.path
    if purpose == "research":
        if path not in RESEARCH_PATHS:
            raise ValueError("research_route_not_allowed")
        credential = research_token(env) if token is None else _token(token)
    elif purpose == "audit":
        if path not in AUDIT_PATHS or data is not None or token is not None:
            raise ValueError("audit_route_not_allowed")
        if mode == "scoped":
            credential = _token(env.get("JIAOYI_AUDIT_BEARER_TOKEN"))
        elif path.startswith("/owner/"):
            credential = _token(env.get("JIAOYI_UI_PASSWORD"))
        else:
            credential = None
    elif purpose == "liveness":
        if route != "/health" or data is not None or token is not None:
            raise ValueError("liveness_route_not_allowed")
        credential = None
    else:
        raise ValueError("invalid_trading_purpose")
    if data is not None and not isinstance(data, bytes):
        raise ValueError("invalid_trading_payload")
    base = origin(base, env)
    headers = {"Accept": "application/json", "Cache-Control": "no-cache"}
    if credential is not None:
        headers["Authorization"] = "Bearer " + credential
    if data is not None:
        headers["Content-Type"] = "application/json"
    result = urllib.request.Request(base + route, data=data, headers=headers,
                                    method="POST" if data is not None else "GET")
    result._jiaoyi_trading_request = True
    result._jiaoyi_private_transport = base.startswith("http:")
    return result


def open_request(req, timeout):
    if not getattr(req, "_jiaoyi_trading_request", False):
        raise ValueError("unvalidated_trading_request")
    handlers = [NoRedirect()]
    if getattr(req, "_jiaoyi_private_transport", False):
        # Do not send a plaintext private-service credential through an ambient
        # HTTP proxy. This does not establish Railway network reachability/TLS.
        handlers.append(urllib.request.ProxyHandler({}))
    return urllib.request.build_opener(*handlers).open(req, timeout=timeout)


def ensure_diagnostic(route, value):
    path = urllib.parse.urlsplit(route).path
    if not isinstance(value, dict) or not value:
        raise ValueError("diagnostic_shape_invalid")
    required = {
        "/health": ("broker_accounts", "trading", "strategy3"),
        "/operations/reports": ("reports",),
        "/owner/setup-assessments": ("accounts",),
        "/operations/setup-assessments": ("accounts",),
    }.get(path)
    if required is not None and any(not isinstance(value.get(k), list) for k in required):
        raise ValueError("diagnostic_evidence_missing")
    if set(value) <= {"status"}:
        raise ValueError("diagnostic_evidence_missing")
    # Shape qualification does not certify account count, freshness or financial
    # readiness. Existing downstream evidence checks retain that responsibility.
    return value
