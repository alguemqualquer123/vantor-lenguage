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

    for line in source.lines() {
        let line = line.trim();

        if line.contains("Window::create") || line.contains("let window = Window") {
            if let Some(win) = current_window.take() {
                if !win.elements.is_empty() {
                    windows.push(win);
                }
            }
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
            if line.contains("setTitle(") || line.contains("title:") {
                win.title = extract_string_value(line);
            }
            if line.contains("setSize(") || line.contains("width:") || line.contains("height:") {
                if let Some((w, h)) = extract_size(line) {
                    win.width = w;
                    win.height = h;
                }
            }
            if line.contains(".show()") || line.contains("window.show()") {
                win.visible = true;
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

    if let Some(win) = current_window {
        if !win.elements.is_empty() {
            windows.push(win);
        }
    }

    windows
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
        extract_quoted_string(rest)
    } else {
        String::new()
    }
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
                                            if ui.button(&item.label).clicked() {
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
