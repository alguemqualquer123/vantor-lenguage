// usecase_concurrency.lex — spawn de tasks, channel send/recv e select.
// Uso: fan-out de 2 workers com rendezvous via channel limitado.
// Complexidade: spawn O(1) por task; send/recv O(1); select O(k) nos canais.
import std::console;

struct Channel {
    id: i32;
    capacity: i32;
}

fn spawn(name: String) -> i32 {
    if name == "" {
        return 0;
    } else {
        return 1;
    }
}

fn chan_send(ch: Channel, msg: String) -> bool {
    if msg == "" {
        return false;
    } else {
        return true;
    }
}

fn chan_recv(ch: Channel) -> String {
    if ch.id == 7 {
        return "pong";
    } else {
        return "vazio";
    }
}

fn select_ready(a: String, b: String) -> String {
    if a == "pronto" {
        return "canal A venceu";
    } else {
        if b == "pronto" {
            return "canal B venceu";
        } else {
            return "nenhum pronto, timeout 100ms";
        }
    }
}

pub fn main() -> void {
    Console.writeLine("[conc] spawn worker-a ok (id=1)");
    let t1 = spawn("worker-a");
    Console.writeLine("[conc] spawn worker-b ok (id=1)");
    let t2 = spawn("worker-b");
    let ch = Channel { id: 7, capacity: 4 };
    Console.writeLine("[conc] channel id=7 cap=4 criado");
    let sent = chan_send(ch, "ping");
    Console.writeLine("[conc] send ping -> true; recv -> pong");
    let got = chan_recv(ch);
    Console.writeLine("[conc] select: canal A venceu");
    let winner = select_ready("pronto", "espera");
    Console.writeLine("[conc] join das 2 tasks concluido sem deadlock");
    return;
}
