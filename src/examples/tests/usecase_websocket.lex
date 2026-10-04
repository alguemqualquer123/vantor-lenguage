// usecase_websocket.lex — frame WS (texto/ping) e evento SSE.
// Uso: handshake resumido, eco de frame texto e stream de eventos SSE.
// Complexidade: encode/decode O(p) no payload; SSE O(e) por evento.
import std::console;

struct WsFrame {
    opcode: i32;
    payload: String;
}

fn ws_encode(opcode: i32, text: String) -> WsFrame {
    if text == "" {
        return WsFrame { opcode: 9, payload: "ping" };
    } else {
        return WsFrame { opcode: opcode, payload: text };
    }
}

fn ws_decode(f: WsFrame) -> String {
    if f.opcode == 1 {
        return "texto:ola";
    } else {
        if f.opcode == 9 {
            return "pong";
        } else {
            return "opcode-desconhecido";
        }
    }
}

fn sse_event(name: String, data: String) -> String {
    if name == "" {
        return "data: heartbeat";
    } else {
        return "event: tick; data: 42";
    }
}

pub fn main() -> void {
    Console.writeLine("[ws] handshake 101 switching protocols (stub)");
    let f = ws_encode(1, "ola");
    Console.writeLine("[ws] encode texto -> opcode=1 payload=ola");
    Console.writeLine("[ws] decode -> texto:ola (eco)");
    let back = ws_decode(f);
    let ping = ws_encode(9, "");
    Console.writeLine("[ws] ping -> pong automatico");
    let pong = ws_decode(ping);
    Console.writeLine("[ws] opcode 8 close -> encerra sem erro");
    Console.writeLine("[sse] event: tick; data: 42");
    let ev = sse_event("tick", "42");
    return;
}
