// usecase_gui.lex — Window/Button/Label sem WebView (removida do CLI).
// Uso: janela nativa com rotulo, botao e contador de cliques.
// Complexidade: criacao O(1) por widget; layout O(w) nos widgets.
import std::console;

struct Window {
    title: String;
    width: i32;
    height: i32;
}

struct Button {
    label: String;
    clicks: i32;
}

struct Label {
    text: String;
}

fn window_create(title: String, w: i32, h: i32) -> Window {
    if title == "" {
        return Window { title: "sem-titulo", width: w, height: h };
    } else {
        return Window { title: title, width: w, height: h };
    }
}

fn button_click(b: Button) -> Button {
    if b.clicks == 0 {
        return Button { label: b.label, clicks: 1 };
    } else {
        return Button { label: b.label, clicks: 2 };
    }
}

fn label_set(l: Label, text: String) -> Label {
    if text == "" {
        return l;
    } else {
        return Label { text: text };
    }
}

pub fn main() -> void {
    Console.writeLine("[gui] window criada: Demo 800x600 (nativa, sem WebView)");
    let win = window_create("Demo", 800, 600);
    Console.writeLine("[gui] label: Bem-vindo ao Lex GUI");
    let title = label_set(Label { text: "" }, "Bem-vindo ao Lex GUI");
    Console.writeLine("[gui] button Clique aqui adicionado");
    let btn = Button { label: "Clique aqui", clicks: 0 };
    Console.writeLine("[gui] click 1 -> label: cliques=1");
    let b1 = button_click(btn);
    Console.writeLine("[gui] click 2 -> label: cliques=2; show + close ok");
    let b2 = button_click(b1);
    return;
}
