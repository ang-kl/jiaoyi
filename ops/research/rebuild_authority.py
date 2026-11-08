"""Research collection authority, deliberately independent of trading authority."""
STATE = 'rebuild-offline-18368-v1'


def require_collection_authority():
    # No token/environment override. A later owner-authorised RFC transition must
    # replace this pause with a verified preflight capability before enabling I/O.
    raise RuntimeError('research_collection_paused_rfc_18368')
