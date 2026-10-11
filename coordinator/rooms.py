"""
Veiled Court — friend rooms

Anonymous two-player games relayed by the coordinator. No accounts, no GPU.
The server only checks seat, turn order and bounds; both clients apply the
shared JS rules (frontend/js/rules.js) to the move list.

  POST /room            {size, client_id, color?} → {code}; creator sits as
                        color (black by default), the friend takes the other
  GET  /room/{code}     the room page (404 page if the room is unknown)
  WS   /room/{code}/ws?cid=<client_id>
"""

import json
import os
import secrets
import time

from fastapi import APIRouter, Request, WebSocket, WebSocketDisconnect
from fastapi.responses import FileResponse, HTMLResponse, JSONResponse

router = APIRouter()

# ── config ──

ALPHABET = "abcdefghjkmnpqrstuvwxyz23456789"  # no i/l/o/0/1
CODE_LEN = 5
SIZES = (9, 13, 19)
MAX_ROOMS = 500
ROOM_TTL = 24 * 3600     # idle rooms expire after a day
MAX_MESSAGE = 512        # bytes; real messages are ~40
MAX_SOCKETS = 32         # per room, players + spectators
MAX_CLIENT_ID = 64

FRONTEND_DIR = os.path.join(os.path.dirname(__file__), "..", "frontend")
if not os.path.isdir(FRONTEND_DIR):
    FRONTEND_DIR = os.path.join(os.getcwd(), "frontend")

# ── state ──

# code -> {size, moves, seats: {black, white}, sockets: [(ws, color)],
#          result, last_activity}
# moves: [{"x": int, "y": int} | {"pass": True}]; black plays even indices.
rooms: dict[str, dict] = {}


def _cleanup():
    """Drop rooms idle for longer than ROOM_TTL with nobody connected."""
    cutoff = time.time() - ROOM_TTL
    for code in [c for c, r in rooms.items() if r["last_activity"] < cutoff and not r["sockets"]]:
        del rooms[code]


def _get_room(code: str):
    _cleanup()
    return rooms.get(code)


def _turn(room) -> str:
    return "black" if len(room["moves"]) % 2 == 0 else "white"


def _presence(room) -> dict:
    connected = {color for _, color in room["sockets"]}
    return {
        "type": "presence",
        "seated": {c: room["seats"][c] is not None for c in ("black", "white")},
        "connected": {c: c in connected for c in ("black", "white")},
    }


async def _broadcast(room, msg: dict):
    for ws, color in list(room["sockets"]):
        try:
            await ws.send_json(msg)
        except Exception:
            if (ws, color) in room["sockets"]:
                room["sockets"].remove((ws, color))


# ── HTTP ──

@router.post("/room")
async def create_room(request: Request):
    try:
        body = await request.json()
    except Exception:
        body = {}
    size = body.get("size", 19) if isinstance(body, dict) else 19
    client_id = body.get("client_id") if isinstance(body, dict) else None
    color = body.get("color", "black") if isinstance(body, dict) else "black"
    if size not in SIZES:
        return JSONResponse({"error": "size must be 9, 13 or 19"}, status_code=400)
    if not isinstance(client_id, str) or not 0 < len(client_id) <= MAX_CLIENT_ID:
        return JSONResponse({"error": "client_id required"}, status_code=400)
    if color not in ("black", "white"):
        return JSONResponse({"error": "color must be black or white"}, status_code=400)

    _cleanup()
    if len(rooms) >= MAX_ROOMS:
        return JSONResponse({"error": "the court is full, try later"}, status_code=503)

    code = "".join(secrets.choice(ALPHABET) for _ in range(CODE_LEN))
    while code in rooms:
        code = "".join(secrets.choice(ALPHABET) for _ in range(CODE_LEN))
    rooms[code] = {
        "size": size,
        "moves": [],
        "seats": {"black": None, "white": None, color: client_id},
        "sockets": [],
        "result": None,
        "last_activity": time.time(),
    }
    return {"code": code}


NOT_FOUND_PAGE = """<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>The Court</title>
<link href="https://fonts.googleapis.com/css2?family=Cormorant+Garamond:ital,wght@0,400;0,600;1,400&display=swap" rel="stylesheet">
<link rel="stylesheet" href="/css/themes.css"><link rel="stylesheet" href="/css/board.css">
<link rel="stylesheet" href="/css/room.css">
</head><body><div id="room-missing" class="room-panel">
<p class="room-line">This room has faded.</p>
<a class="room-link" href="/index.html">return to the court</a>
</div></body></html>"""


@router.get("/room/{code}")
async def room_page(code: str):
    if _get_room(code) is None:
        return HTMLResponse(NOT_FOUND_PAGE, status_code=404)
    return FileResponse(os.path.join(FRONTEND_DIR, "room.html"), media_type="text/html")


# ── WebSocket ──

@router.websocket("/room/{code}/ws")
async def room_ws(ws: WebSocket, code: str):
    await ws.accept()
    room = _get_room(code)
    if room is None:
        await ws.send_json({"type": "error", "reason": "not_found"})
        await ws.close(code=4404)
        return
    if len(room["sockets"]) >= MAX_SOCKETS:
        await ws.send_json({"type": "error", "reason": "full"})
        await ws.close(code=4429)
        return

    # Seat by client id: the creator has their chosen seat, the first other
    # client takes the empty one, everyone else watches. A reconnect with the
    # same id gets the same seat.
    cid = ws.query_params.get("cid", "")[:MAX_CLIENT_ID]
    seats = room["seats"]
    if cid and cid == seats["black"]:
        color = "black"
    elif cid and cid == seats["white"]:
        color = "white"
    elif cid and None in seats.values():
        color = "white" if seats["white"] is None else "black"
        seats[color] = cid
    else:
        color = "spectator"

    entry = (ws, color)
    room["sockets"].append(entry)
    room["last_activity"] = time.time()
    await ws.send_json({
        "type": "state",
        "size": room["size"],
        "moves": room["moves"],
        "you": color,
        "result": room["result"],
        **{k: v for k, v in _presence(room).items() if k != "type"},
    })
    await _broadcast(room, _presence(room))

    try:
        while True:
            raw = await ws.receive_text()
            room["last_activity"] = time.time()
            error = await _handle(room, color, raw)
            if error:
                await ws.send_json({"type": "error", "reason": error})
    except WebSocketDisconnect:
        pass
    except Exception:
        pass
    finally:
        if entry in room["sockets"]:
            room["sockets"].remove(entry)
        room["last_activity"] = time.time()
        await _broadcast(room, _presence(room))


async def _handle(room, color: str, raw: str):
    """Apply one client message. Returns an error reason, or None."""
    if len(raw) > MAX_MESSAGE:
        return "too_large"
    try:
        msg = json.loads(raw)
    except ValueError:
        return "bad_json"
    if not isinstance(msg, dict):
        return "bad_message"

    kind = msg.get("type")
    if kind == "ping":
        return None
    if color == "spectator":
        return "spectator"
    if room["result"]:
        return "game_over"

    if kind == "resign":
        winner = "white" if color == "black" else "black"
        room["result"] = {"reason": "resign", "winner": winner}
        await _broadcast(room, {"type": "end", "result": room["result"]})
        return None

    if kind not in ("move", "pass"):
        return "bad_message"
    if color != _turn(room):
        return "not_your_turn"
    # Optional move index guards against a resend after reconnect
    n = msg.get("n")
    if n is not None and n != len(room["moves"]):
        return "stale"

    if kind == "move":
        x, y = msg.get("x"), msg.get("y")
        size = room["size"]
        if type(x) is not int or type(y) is not int or not (0 <= x < size and 0 <= y < size):
            return "out_of_bounds"
        move = {"x": x, "y": y}
    else:
        move = {"pass": True}

    room["moves"].append(move)
    await _broadcast(room, {"type": "move", "n": len(room["moves"]) - 1, **move})

    moves = room["moves"]
    if len(moves) >= 2 and moves[-1].get("pass") and moves[-2].get("pass"):
        room["result"] = {"reason": "passes"}
        await _broadcast(room, {"type": "end", "result": room["result"]})
    return None
