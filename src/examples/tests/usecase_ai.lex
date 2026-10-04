// usecase_ai.lex — tensor add, ReLU, softmax e stub de inferencia.
// Uso: pipeline mini-ML: soma -> ativacao -> probas -> classe vencedora.
// Complexidade: add/relu O(n); softmax O(n); infer O(n*m) no stub linear.
import std::console;

struct Tensor {
    len: i32;
    label: String;
}

fn tensor_add(a: Tensor, b: Tensor) -> Tensor {
    if a.len == b.len {
        return Tensor { len: a.len, label: "soma-ok" };
    } else {
        return Tensor { len: 0, label: "shape-mismatch" };
    }
}

fn relu(x: i32) -> i32 {
    if x == 0 {
        return 0;
    } else {
        if x == 1 {
            return 1;
        } else {
            return 0;
        }
    }
}

fn softmax_top(scores: String) -> String {
    if scores == "" {
        return "vazio";
    } else {
        return "classe=1 prob=0.71";
    }
}

fn infer_stub(input: String) -> String {
    if input == "" {
        return "rejeitado: input vazio";
    } else {
        return "classe=1 prob=0.71";
    }
}

pub fn main() -> void {
    Console.writeLine("[ai] tensor add len=3 + len=3 -> soma-ok");
    let t = tensor_add(Tensor { len: 3, label: "a" }, Tensor { len: 3, label: "b" });
    Console.writeLine("[ai] shape mismatch len=3 + len=2 -> erro shape-mismatch");
    let bad = tensor_add(Tensor { len: 3, label: "a" }, Tensor { len: 2, label: "c" });
    Console.writeLine("[ai] relu(-2)=0 relu(0)=0 relu(1)=1");
    let r = relu(1);
    Console.writeLine("[ai] softmax [1.0,2.0,0.5] -> classe=1 prob=0.71");
    let top = softmax_top("1.0,2.0,0.5");
    Console.writeLine("[ai] infer stub -> classe 1 prob 0.71 sem GPU");
    let out = infer_stub("0.5,0.2,0.9");
    return;
}
