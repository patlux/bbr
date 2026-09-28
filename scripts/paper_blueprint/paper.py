"""Minimal client for the local Paper Desktop MCP endpoint (same server the
pi MCP proxy uses): initialize once, then call tools. Session id cached in
/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/.session."""
import json
import os
import sys
import urllib.request

URL = "http://127.0.0.1:29979/mcp"
FILE = "01KZY8FRMDSAYFJ4TJ9TYTV4AF"
SESSION_PATH = "/Users/patwoz/dev/Personal/macos/bbr/scripts/paper_blueprint/.session"
_id = [0]
_wait = [0]


def _post(payload, session=None):
    data = json.dumps(payload).encode()
    headers = {
        "Content-Type": "application/json",
        "Accept": "application/json, text/event-stream",
    }
    if session:
        headers["Mcp-Session-Id"] = session
    req = urllib.request.Request(URL, data=data, headers=headers, method="POST")
    with urllib.request.urlopen(req, timeout=120) as resp:
        sid = resp.headers.get("Mcp-Session-Id")
        body = resp.read().decode()
        ctype = resp.headers.get("Content-Type", "")
    if "event-stream" in ctype:
        msgs = []
        for line in body.splitlines():
            if line.startswith("data:"):
                chunk = line[5:].strip()
                if chunk:
                    msgs.append(json.loads(chunk))
        result = msgs[-1] if msgs else None
    else:
        result = json.loads(body) if body.strip() else None
    return sid, result


def session():
    if os.path.exists(SESSION_PATH):
        return open(SESSION_PATH).read().strip()
    return init()


def init():
    _id[0] += 1
    sid, res = _post({
        "jsonrpc": "2.0", "id": _id[0], "method": "initialize",
        "params": {"protocolVersion": "2025-03-26", "capabilities": {},
                   "clientInfo": {"name": "bbr-blueprint", "version": "1"}},
    })
    _post({"jsonrpc": "2.0", "method": "notifications/initialized"}, sid)
    if sid:
        open(SESSION_PATH, "w").write(sid)
    return sid


def call(tool, args, retry=True):
    args = dict(args)
    args.setdefault("fileId", FILE)
    _id[0] += 1
    payload = {"jsonrpc": "2.0", "id": _id[0], "method": "tools/call",
               "params": {"name": tool, "arguments": args}}
    try:
        _, res = _post(payload, session())
    except urllib.error.HTTPError as e:
        if retry and e.code in (400, 404):
            if os.path.exists(SESSION_PATH):
                os.remove(SESSION_PATH)
            init()
            return call(tool, args, retry=False)
        raise
    if res is None:
        raise RuntimeError("empty response")
    if "error" in res:
        if retry and "session" in json.dumps(res["error"]).lower():
            os.remove(SESSION_PATH)
            init()
            return call(tool, args, retry=False)
        raise RuntimeError(json.dumps(res["error"]))
    content = res["result"].get("content", [])
    if res["result"].get("isError"):
        message = " ".join(c.get("text", "") for c in content)
        if "disconnected" in message and _wait[0] < 60:
            import time
            _wait[0] += 1
            time.sleep(10)
            return call(tool, args, retry)
        raise RuntimeError(message)
    _wait[0] = 0
    texts = [c.get("text", "") for c in content if c.get("type") == "text"]
    # The first text block is the file header; the payload is the last.
    last = texts[-1] if texts else ""
    try:
        return json.loads(last)
    except json.JSONDecodeError:
        return last


def shot(node_id, path, scale=1):
    """Save a node screenshot (PNG/JPEG bytes) to `path`."""
    import base64
    _id[0] += 1
    payload = {"jsonrpc": "2.0", "id": _id[0], "method": "tools/call",
               "params": {"name": "get_screenshot",
                          "arguments": {"fileId": FILE, "nodeId": node_id, "scale": scale}}}
    _, res = _post(payload, session())
    for c in res["result"].get("content", []):
        if c.get("type") == "image":
            open(path, "wb").write(base64.b64decode(c["data"]))
            return path
    raise RuntimeError(json.dumps(res)[:500])


if __name__ == "__main__":
    if sys.argv[1] == "shot":
        print(shot(sys.argv[2], sys.argv[3], float(sys.argv[4]) if len(sys.argv) > 4 else 1))
        sys.exit(0)
    tool = sys.argv[1]
    args = json.loads(sys.argv[2]) if len(sys.argv) > 2 else {}
    print(json.dumps(call(tool, args), indent=1)[:4000])
