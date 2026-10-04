use eframe::egui;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

static GUI_STATE: Lazy<Mutex<GuiState>> = Lazy::new(|| Mutex::new(GuiState::default()));

#[derive(Default)]
pub struct GuiState {
    pub windows: Vec<WindowState>,
    pub current_window: Option<usize>,
    pub events: Vec<GuiEvent>,
    pub output: String,
}

#[derive(Clone)]
pub struct WindowState {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub elements: Vec<Element>,
    pub visible: bool,
}

#[derive(Clone)]
pub enum Element {
    Label {
        text: String,
        id: String,
    },
    Button {
        text: String,
        id: String,
        onclick: Option<String>,
    },
    TextField {
        id: String,
        placeholder: String,
        text: String,
    },
    TextArea {
        id: String,
        text: String,
        rows: usize,
    },
    Checkbox {
        text: String,
        id: String,
        checked: bool,
    },
    ComboBox {
        id: String,
        items: Vec<String>,
        selected: usize,
    },
    ListView {
        id: String,
        items: Vec<String>,
    },
    Image {
        id: String,
        path: String,
    },
    Canvas {
        id: String,
        width: f32,
        height: f32,
    },
    MenuBar {
        id: String,
        menus: Vec<Menu>,
    },
    Widget {
        id: String,
        element_type: String,
        properties: HashMap<String, String>,
    },
}

#[derive(Clone)]
pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
}

#[derive(Clone)]
pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub action: Option<String>,
}

#[derive(Clone)]
pub enum GuiEvent {
    Click { id: String },
    Change { id: String, value: String },
    Submit { id: String, value: String },
    Select { id: String, index: usize },
    WindowClose,
    WindowResize { width: f32, height: f32 },
}

pub fn parse_gui_code(source: &str) -> Vec<WindowState> {
    let mut windows = Vec::new();
    let mut current_window: Option<WindowState> = None;
    let mut element_id_counter = 0;
    let mut menubar_counter = 0;
    // Menu under construction + finished menus of the current window.
    // `lex gui` is line-oriented: `Menu::new` starts a menu, `MenuItem::new`
    // appends to it, and everything is flushed into the window on the next
    // `Window::create` (or end of source). Wiring lines (`menu.add`,
    // `bar.add`, `setMenuBar`) need no handling.
    let mut current_menu: Option<Menu> = None;
    let mut current_menubar: Vec<Menu> = Vec::new();

    for line in source.lines() {
        let line = line.trim();

        if line.contains("Window::create") || line.contains("let window = Window") {
            if let Some(mut win) = current_window.take() {
                flush_menu(&mut current_menu, &mut current_menubar);
                if !current_menubar.is_empty() {
                    menubar_counter += 1;
                    win.elements.push(Element::MenuBar {
                        id: format!("menubar_{}", menubar_counter),
                        menus: std::mem::take(&mut current_menubar),
                    });
                }
                if !win.elements.is_empty() {
                    windows.push(win);
                }
            }
            current_menu = None;
            let title = extract_title_from_line(line);
            current_window = Some(WindowState {
                title,
                width: 600.0,
                height: 400.0,
                elements: Vec::new(),
                visible: true,
            });
        }

        if let Some(ref mut win) = current_window {
            // Struct-literal config lines (`Cfg { title: ..., width: ... }`)
            // belong to a *future* `Window::create`, not to the window being
            // built — only the create line itself may carry `{...}` here.
            let is_struct_config = line.contains('{') && !line.contains("Window::create");
            if !is_struct_config {
                if line.contains("setTitle(") || line.contains("title:") {
                    win.title = extract_string_value(line);
                }
                if line.contains("setSize(") || line.contains("width:") || line.contains("height:") {
                    if let Some((w, h)) = extract_size(line) {
                        win.width = w;
                        win.height = h;
                    }
                }
            }
            if line.contains(".show()") || line.contains("window.show()") {
                win.visible = true;
            }
        }

        // `MenuBar::new()` opens the window's menu bar (idempotent).
        if line.contains("MenuBar::new") {
            flush_menu(&mut current_menu, &mut current_menubar);
        }

        if line.contains("Menu::new(") && !line.contains("MenuItem") && !line.contains("MenuBar") {
            flush_menu(&mut current_menu, &mut current_menubar);
            current_menu = Some(Menu {
                title: extract_nth_quoted(line, 0),
                items: Vec::new(),
            });
        }

        if line.contains("MenuItem::new") {
            if current_menu.is_none() {
                current_menu = Some(Menu {
                    title: "Menu".to_string(),
                    items: Vec::new(),
                });
            }
            if let Some(ref mut menu) = current_menu {
                menu.items.push(MenuItem {
                    label: extract_nth_quoted(line, 0),
                    shortcut: match extract_nth_quoted(line, 1) {
                        s if s.is_empty() => None,
                        s => Some(s),
                    },
                    action: None,
                });
            }
        }

        if let Some(ref mut win) = current_window {
            if line.contains("Label::create") {
                let text = extract_string_value(line);
                element_id_counter += 1;
                let id = format!("label_{}", element_id_counter);
                win.elements.push(Element::Label { text, id });
            }

            if line.contains("Button::create") {
                let text = extract_string_value(line);
                element_id_counter += 1;
                let id = format!("button_{}", element_id_counter);
                let onclick = extract_closure(line);
                win.elements.push(Element::Button { text, id, onclick });
            }

            if line.contains("TextField::create") {
                element_id_counter += 1;
                let id = format!("textfield_{}", element_id_counter);
                let placeholder = extract_placeholder(line);
                win.elements.push(Element::TextField {
                    id,
                    placeholder,
                    text: String::new(),
                });
            }

            // `field.setPlaceholder("...")` attaches to the last TextField.
            if line.contains("setPlaceholder(") {
                let ph = extract_quoted_string(line);
                if !ph.is_empty() {
                    if let Some(el) = win
                        .elements
                        .iter_mut()
                        .rfind(|e| matches!(e, Element::TextField { .. }))
                    {
                        if let Element::TextField { placeholder, .. } = el {
                            *placeholder = ph;
                        }
                    }
                }
            }

            if line.contains("TextArea::create") {
                element_id_counter += 1;
                let id = format!("textarea_{}", element_id_counter);
                let rows = extract_rows(line);
                win.elements.push(Element::TextArea {
                    id,
                    text: String::new(),
                    rows,
                });
            }

            if line.contains("Checkbox::create") {
                let text = extract_string_value(line);
                element_id_counter += 1;
                let id = format!("checkbox_{}", element_id_counter);
                win.elements.push(Element::Checkbox {
                    text,
                    id,
                    checked: false,
                });
            }

            if line.contains("ComboBox::new") || line.contains("ComboBox::create") {
                element_id_counter += 1;
                let id = format!("combobox_{}", element_id_counter);
                win.elements.push(Element::ComboBox {
                    id,
                    items: Vec::new(),
                    selected: 0,
                });
            }

            if line.contains("ListView::new") {
                element_id_counter += 1;
                let id = format!("listview_{}", element_id_counter);
                win.elements.push(Element::ListView {
                    id,
                    items: Vec::new(),
                });
            }
        }
    }

    if let Some(mut win) = current_window.take() {
        flush_menu(&mut current_menu, &mut current_menubar);
        if !current_menubar.is_empty() {
            menubar_counter += 1;
            win.elements.push(Element::MenuBar {
                id: format!("menubar_{}", menubar_counter),
                menus: std::mem::take(&mut current_menubar),
            });
        }
        if !win.elements.is_empty() {
            windows.push(win);
        }
    }

    windows
}

/// Move the menu under construction into the window's menu bar.
fn flush_menu(current_menu: &mut Option<Menu>, current_menubar: &mut Vec<Menu>) {
    if let Some(menu) = current_menu.take() {
        current_menubar.push(menu);
    }
}

/// n-th `"quoted"` string on the line (0-based), or `""`.
fn extract_nth_quoted(line: &str, n: usize) -> String {
    let mut rest = line;
    let mut idx = 0;
    while let Some(start) = rest.find('"') {
        rest = &rest[start + 1..];
        if let Some(end) = rest.find('"') {
            if idx == n {
                return rest[..end].to_string();
            }
            idx += 1;
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    String::new()
}

fn extract_title_from_line(line: &str) -> String {
    if let Some(start) = line.find("title:") {
        let rest = &line[start + 6..];
        extract_quoted_string(rest)
    } else {
        "Lexicon App".to_string()
    }
}

fn extract_string_value(line: &str) -> String {
    extract_quoted_string(line)
}

fn extract_quoted_string(s: &str) -> String {
    let s = s.trim();
    if let Some(start) = s.find('"') {
        if let Some(end) = s[start + 1..].find('"') {
            return s[start + 1..start + 1 + end].to_string();
        }
    }
    String::new()
}

fn extract_size(line: &str) -> Option<(f32, f32)> {
    let nums: Vec<f32> = line
        .split(|c: char| !c.is_numeric() && c != '.')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    if nums.len() >= 2 {
        Some((nums[0], nums[1]))
    } else if nums.len() == 1 {
        Some((nums[0], 400.0))
    } else {
        None
    }
}

fn extract_placeholder(line: &str) -> String {
    if let Some(start) = line.find("Placeholder") {
        let rest = &line[start..];
        let ph = extract_quoted_string(rest);
        if !ph.is_empty() {
            return ph;
        }
    }
    // Fallback: `TextField::create("Seu nome")` carries its own hint.
    extract_quoted_string(line)
}

fn extract_rows(line: &str) -> usize {
    if let Some(start) = line.find("setRows(") {
        let rest = &line[start + 8..];
        rest.chars()
            .take_while(|c| c.is_numeric())
            .collect::<String>()
            .parse()
            .unwrap_or(4)
    } else {
        4
    }
}

fn extract_closure(line: &str) -> Option<String> {
    if let Some(start) = line.find("fn(") {
        if let Some(end) = line[start..].find(')') {
            return Some(line[start..start + end + 1].to_string());
        }
    }
    None
}

pub fn run_gui(source: &str) {
    let windows = parse_gui_code(source);

    if windows.is_empty() {
        println!("No GUI windows defined in the code.");
        return;
    }

    println!("Starting GUI with {} window(s)...", windows.len());

    let app = LexiconGuiApp::new(windows);

    eframe::run_native(
        "Lexicon GUI",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .ok();
}

struct LexiconGuiApp {
    windows: Vec<WindowState>,
    current_window: usize,
    input_values: HashMap<String, String>,
    output: String,
}

impl LexiconGuiApp {
    fn new(windows: Vec<WindowState>) -> Self {
        Self {
            windows,
            current_window: 0,
            input_values: HashMap::new(),
            output: String::new(),
        }
    }
}

impl eframe::App for LexiconGuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.windows.is_empty() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("No windows defined");
            });
            return;
        }

        let window = &mut self.windows[self.current_window];

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&window.title);
            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for element in &window.elements {
                    match element {
                        Element::Label { text, .. } => {
                            ui.label(text);
                            ui.add_space(5.0);
                        }
                        Element::Button {
                            text,
                            id,
                            onclick: _,
                        } => {
                            if ui.button(text).clicked() {
                                self.output.push_str(&format!("Button clicked: {}\n", text));
                                println!("Button clicked: {}", text);
                            }
                            ui.add_space(5.0);
                        }
                        Element::TextField {
                            id,
                            placeholder,
                            text: _,
                        } => {
                            let input = self
                                .input_values
                                .entry(id.clone())
                                .or_insert_with(String::new);
                            ui.label(placeholder);
                            ui.text_edit_singleline(input);
                            ui.add_space(5.0);
                        }
                        Element::TextArea { id, text: _, rows } => {
                            let input = self
                                .input_values
                                .entry(id.clone())
                                .or_insert_with(String::new);
                            ui.label("Text:");
                            egui::TextEdit::multiline(input)
                                .desired_rows(*rows)
                                .show(ui);
                            ui.add_space(5.0);
                        }
                        Element::Checkbox { text, id, checked } => {
                            let mut check = *checked;
                            ui.checkbox(&mut check, text);
                            ui.add_space(5.0);
                        }
                        Element::ComboBox {
                            id,
                            items,
                            selected: _,
                        } => {
                            ui.label("ComboBox:");
                            for (i, item) in items.iter().enumerate() {
                                ui.label(format!("  {}. {}", i + 1, item));
                            }
                            ui.add_space(5.0);
                        }
                        Element::ListView { id, items } => {
                            ui.label("List:");
                            for item in items {
                                ui.label(format!("  • {}", item));
                            }
                            ui.add_space(5.0);
                        }
                        Element::Image { id, path } => {
                            ui.label(format!("[Image: {}]", path));
                            ui.add_space(5.0);
                        }
                        Element::Canvas { id, width, height } => {
                            let (rect, _response) = ui.allocate_exact_size(
                                egui::vec2(*width, *height),
                                egui::Sense::click(),
                            );
                            ui.painter().rect_filled(
                                rect,
                                0.0,
                                egui::Color32::from_rgb(240, 240, 240),
                            );
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "[Canvas]",
                                egui::FontId::default(),
                                egui::Color32::GRAY,
                            );
                            ui.add_space(5.0);
                        }
                        Element::MenuBar { id, menus } => {
                            egui::menu::bar(ui, |ui| {
                                for menu in menus {
                                    ui.menu_button(&menu.title, |ui| {
                                        for item in &menu.items {
                                            let caption = match &item.shortcut {
                                                Some(s) => format!("{}  ({})", item.label, s),
                                                None => item.label.clone(),
                                            };
                                            if ui.button(&caption).clicked() {
                                                self.output
                                                    .push_str(&format!("Menu: {}\n", item.label));
                                                ui.close_menu();
                                            }
                                        }
                                    });
                                }
                            });
                        }
                        Element::Widget {
                            id,
                            element_type,
                            properties: _,
                        } => {
                            ui.label(format!("[{}]", element_type));
                            ui.add_space(5.0);
                        }
                    }
                }
            });
        });

        if !self.output.is_empty() {
            egui::Window::new("Output").show(ctx, |ui| {
                ui.label(&self.output);
            });
        }
    }
}

#[cfg(test)]
mod gui_parse_tests {
    use super::*;

    const DEMO: &str = r#"
pub fn main() -> void {
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

    let titulo = Label::create("Bem-vindo!");
    window.add(titulo);
    let nome = TextField::create("Seu nome");
    nome.setPlaceholder("Digite seu nome");
    window.add(nome);
    let lembrete = Checkbox::create("Lembrar de mim");
    window.add(lembrete);
    let botao = Button::create("Clique aqui");
    window.add(botao);
    window.show();

    let cfg2 = WinConfig { title: "Sobre", width: 400, height: 300 };
    let sobre = Window::create(cfg2);
    sobre.setSize(400, 300);
    sobre.setTitle("Sobre");
    let info = Label::create("v1.0");
    sobre.add(info);
    sobre.show();
    return;
}
"#;

    #[test]
    fn two_windows_menu_widgets_and_config_isolation() {
        let wins = parse_gui_code(DEMO);
        assert_eq!(wins.len(), 2, "expected 2 windows, got {}", wins.len());

        // Window 1 keeps its own title/size (cfg2 must not leak into it).
        assert_eq!(wins[0].title, "Minha Janela");
        assert_eq!((wins[0].width, wins[0].height), (800.0, 600.0));

        // Menu bar with two menus, labels and shortcuts.
        let mb = wins[0]
            .elements
            .iter()
            .find_map(|e| match e {
                Element::MenuBar { menus, .. } => Some(menus),
                _ => None,
            })
            .expect("window 1 must have a MenuBar");
        assert_eq!(mb.len(), 2);
        assert_eq!(mb[0].title, "Arquivo");
        assert_eq!(mb[0].items.len(), 2);
        assert_eq!(mb[0].items[0].label, "Novo");
        assert_eq!(mb[0].items[0].shortcut.as_deref(), Some("Ctrl+N"));
        assert_eq!(mb[0].items[1].label, "Sair");
        assert_eq!(mb[1].title, "Ajuda");
        assert_eq!(mb[1].items.len(), 1);
        assert_eq!(mb[1].items[0].label, "Sobre");
        assert!(mb[1].items[0].shortcut.is_none());

        // Widgets.
        let labels = wins[0]
            .elements
            .iter()
            .filter(|e| matches!(e, Element::Label { .. }))
            .count();
        let buttons = wins[0]
            .elements
            .iter()
            .filter(|e| matches!(e, Element::Button { .. }))
            .count();
        assert_eq!(labels, 1);
        assert_eq!(buttons, 1);
        let ph = wins[0]
            .elements
            .iter()
            .find_map(|e| match e {
                Element::TextField { placeholder, .. } => Some(placeholder.clone()),
                _ => None,
            })
            .expect("textfield");
        assert_eq!(ph, "Digite seu nome");
        assert!(wins[0]
            .elements
            .iter()
            .any(|e| matches!(e, Element::Checkbox { .. })));

        // Window 2 intact.
        assert_eq!(wins[1].title, "Sobre");
        assert_eq!((wins[1].width, wins[1].height), (400.0, 300.0));
        assert!(wins[1]
            .elements
            .iter()
            .any(|e| matches!(e, Element::Label { .. })));
    }

    #[test]
    fn empty_window_is_dropped() {
        let wins = parse_gui_code(
            "pub fn main() -> void { let w = Window::create(c); w.show(); return; }",
        );
        assert!(wins.is_empty());
    }
}
