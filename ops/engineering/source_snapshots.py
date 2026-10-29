"""Independent bounded public reads with fixed, credential-free receipts."""
import concurrent.futures
import datetime as dt
import http.client
import json
import time
import urllib.error
import urllib.request

ROUTES = ("/health", "/operations/reports")


def fetch(base_url, timeout=8, maxbytes=2 * 1024 * 1024):
    """Return health, operations and per-route receipts; never remote error text."""
    if type(maxbytes) is not int or maxbytes < 1:
        raise ValueError("positive snapshot byte limit required")

    def read(route):
        started = time.monotonic()
        receipt = {"route": route, "ok": False, "http_status": None,
                   "error_code": None, "bytes": 0}
        data = None
        try:
            request = urllib.request.Request(base_url.rstrip("/") + route,
                                            headers={"Cache-Control": "no-cache"})
            with urllib.request.urlopen(request, timeout=timeout) as response:
                status = getattr(response, "status", None)
                receipt["http_status"] = status if type(status) is int and 100 <= status <= 599 else None
                body = response.read(maxbytes + 1)
            receipt["bytes"] = len(body)
            if len(body) > maxbytes:
                receipt["error_code"] = "oversize"
            else:
                try:
                    value = json.loads(body)
                except (ValueError, UnicodeDecodeError, RecursionError):
                    receipt["error_code"] = "invalid_json"
                else:
                    if isinstance(value, dict):
                        data, receipt["ok"] = value, True
                    else:
                        receipt["error_code"] = "invalid_shape"
        except urllib.error.HTTPError as error:
            receipt["error_code"] = "http"
            receipt["http_status"] = error.code if type(error.code) is int and 100 <= error.code <= 599 else None
        except TimeoutError:
            receipt["error_code"] = "timeout"
        except urllib.error.URLError as error:
            receipt["error_code"] = "timeout" if isinstance(error.reason, TimeoutError) else "transport"
        except (OSError, http.client.HTTPException, ValueError, TypeError):
            receipt["error_code"] = "transport"
        finally:
            receipt["observed_at"] = dt.datetime.now(dt.timezone.utc).isoformat()
            receipt["elapsed_ms"] = max(0, round((time.monotonic() - started) * 1000))
        return data, receipt

    with concurrent.futures.ThreadPoolExecutor(max_workers=2) as pool:
        values = list(pool.map(read, ROUTES))
    return values[0][0], values[1][0], {route: item[1] for route, item in zip(ROUTES, values)}
