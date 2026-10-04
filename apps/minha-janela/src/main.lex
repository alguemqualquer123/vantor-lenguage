import core::gui::Window;
import core::gui::Label;
import core::gui::Button;
import core::gui::TextField;
import core::gui::Checkbox;
import core::gui::MenuBar;
import core::gui::Menu;
import core::gui::MenuItem;

struct WinConfig {
    title: String;
    width: int;
    height: int;
}

pub fn main() -> void {
    // --- Janela 1: principal ---
    let cfg = WinConfig { title: "Minha Janela", width: 800, height: 600 };
    let window = Window::create(cfg);
    window.setSize(800, 600);
    window.setTitle("Minha Janela");

    let bar = MenuBar::new();
    let mArquivo = Menu::new("Arquivo");
    let iNovo = MenuItem::new("Novo", "Ctrl+N");
    mArquivo.add(iNovo);
    let iSair = MenuItem::new("Sair", "Ctrl+Q");
    mArquivo.add(iSair);
    bar.add(mArquivo);
    let mAjuda = Menu::new("Ajuda");
    let iSobre = MenuItem::new("Sobre");
    mAjuda.add(iSobre);
    bar.add(mAjuda);
    window.setMenuBar(bar);

    let titulo = Label::create("Bem-vindo ao meu programa!");
    window.add(titulo);

    let nome = TextField::create("Seu nome");
    nome.setPlaceholder("Digite seu nome");
    window.add(nome);

    let lembrete = Checkbox::create("Lembrar de mim");
    window.add(lembrete);

    let botao = Button::create("Clique aqui");
    window.add(botao);
    let botaoFechar = Button::create("Fechar");
    window.add(botaoFechar);

    window.show();

    // --- Janela 2: sobre ---
    let cfg2 = WinConfig { title: "Sobre", width: 400, height: 300 };
    let sobre = Window::create(cfg2);
    sobre.setSize(400, 300);
    sobre.setTitle("Sobre");
    let info = Label::create("Meu programa v1.0");
    sobre.add(info);
    sobre.show();
    return;
}
