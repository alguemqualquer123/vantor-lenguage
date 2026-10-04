//! Native graphics runtime (Spec §"gui"): janelas REAIS desenhadas pelo
//! renderer **wgpu** do eframe — backends Vulkan / DirectX 12 / OpenGL
//! (Metal no macOS; o mesmo motor gráfico do Firefox).
//!
//! # Arquitetura (1 event loop → N viewports)
//!
//! O winit 0.30 permite **um único `EventLoop` por processo**
//! (`EVENT_LOOP_CREATED` é global: uma segunda `build()` devolve
//! `RecreationAttempt`). Por isso existe **um host só**: a primeira
//! `Window::create` sobe uma thread de UI com `eframe::run_native` (loop
//! oculto) e cada janela Lex é uma **viewport egui** desenhada por frame:
//!
//! - Desenho: fila `Cmd` por janela (interpretador → host), drenada 1×/frame.
//! - Entrada/estado: `UiShared` por janela (host → interpretador, sob mutex).
//! - `Window::present(win)` bloqueia até o host pintar a janela (pacing ~60fps).
//! - `main()` terminando com janelas abertas: `wait_windows()` mantém o
//!   processo vivo até todas fecharem (apps orientados a eventos).
//! - `LEXICON_GUI_AUTOQUIT=<frames>` fecha sozinho (CI: `lex run --ci`).
//!
//! Limitação: o event loop precisa da main thread em macOS (usar `lex gui`);
//! Windows/Linux usam `EventLoopBuilderExtWindows::with_any_thread`.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

use eframe::egui;

// ---------------------------------------------------------------------------
// Handles & estado
// ---------------------------------------------------------------------------

/// Widgets criados por Lex (`Button::create`, `Menu::new`, ...).
#[derive(Clone, Debug)]
pub enum Widget {
    Label { text: String },
    Button { label: String },
    TextField { placeholder: String },
    Checkbox { label: String },
    MenuItem { label: String, shortcut: String },
    Menu { title: String },
    MenuBar,
}

/// Registro global de widgets + árvore de menus + layout por janela.
#[derive(Default)]
pub struct Store {
    next_id: i64,
    widgets: HashMap<i64, Widget>,
    /// `menu_bar -> menus`, `menu -> items` (ordem de inserção).
    children: HashMap<i64, Vec<i64>>,
    /// Janela → widgets na ordem em que `window.add` foi chamado.
    window_widgets: HashMap<i64, Vec<i64>>,
    window_menubar: HashMap<i64, i64>,
    /// Valores iniciais de TextField/Checkbox antes de a UI assumir.
    defaults_s: HashMap<i64, String>,
    defaults_b: HashMap<i64, bool>,
}

impl Store {
    fn alloc(&mut self, w: Widget) -> i64 {
        self.next_id += 1;
        let id = self.next_id;
        self.widgets.insert(id, w);
        id
    }
}

fn store() -> &'static Mutex<Store> {
    static S: OnceLock<Mutex<Store>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(Store::default()))
}

/// Estado publicado pelo host a cada frame de cada janela.
#[derive(Default)]
pub struct UiState {
    pub should_close: bool,
    pub frame_done: bool,
    /// A viewport já morreu (fechada/erro/autoquit) — não há mais frames.
    pub closed_done: bool,
    pub keys: HashSet<String>,
    pub mouse: (f32, f32),
    pub mouse_down: bool,
    pub clicked: Vec<i64>,
    pub values: HashMap<i64, String>,
    pub checks: HashMap<i64, bool>,
    pub width: f32,
    pub height: f32,
    pub backend: String,
    /// Cor de fundo da janela (Canvas::clear).
    pub bg: [f32; 4],
    frames: u64,
    autoquit: Option<u64>,
    close_sent: bool,
}

pub struct UiShared {
    st: Mutex<UiState>,
    cv: Condvar,
}

impl UiShared {
    fn new(width: f32, height: f32, autoquit: Option<u64>) -> Self {
        Self {
            st: Mutex::new(UiState {
                width,
                height,
                autoquit,
                bg: [0.06, 0.07, 0.12, 1.0],
                ..Default::default()
            }),
            cv: Condvar::new(),
        }
    }
}

struct WindowHandle {
    tx: Sender<Cmd>,
    ui: Arc<UiShared>,
    title: String,
    width: f32,
    height: f32,
    /// A viewport já foi criada pelo host?
    spawned: bool,
}

fn windows() -> &'static Mutex<HashMap<i64, WindowHandle>> {
    static W: OnceLock<Mutex<HashMap<i64, WindowHandle>>> = OnceLock::new();
    W.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Receptores por janela (o host drena 1× por frame; não cabem no handle
/// porque `send()` fica do lado do interpretador).
fn receivers() -> &'static Mutex<HashMap<i64, Receiver<Cmd>>> {
    static R: OnceLock<Mutex<HashMap<i64, Receiver<Cmd>>>> = OnceLock::new();
    R.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Quadro de uma janela: o que está publicado (sendo desenhado) e o que o
/// interpretador já enviou do quadro seguinte.
#[derive(Default)]
struct Cena {
    publicada: Vec<Cmd>,
    pendente: Vec<Cmd>,
}

/// Último quadro enviado por janela, redesenhado em toda repintura.
fn scenes() -> &'static Mutex<HashMap<i64, Cena>> {
    static S: OnceLock<Mutex<HashMap<i64, Cena>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_window_id() -> i64 {
    static NEXT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
    NEXT.fetch_add(1, Ordering::Relaxed) + 1
}

/// Comandos de desenho interpretador → host (consumidos 1× por frame).
#[derive(Clone)]
enum Cmd {
    Rect { x: f32, y: f32, w: f32, h: f32, c: [f32; 4] },
    Circle { x: f32, y: f32, r: f32, c: [f32; 4] },
    Line { x1: f32, y1: f32, x2: f32, y2: f32, w: f32, c: [f32; 4] },
    Text { x: f32, y: f32, s: String, size: f32, c: [f32; 4] },
    SetTitle(String),
    SetSize(f32, f32),
    /// Fronteira de quadro do programa Lex (`Window::present`).
    Present,
    Close,
}

// ---------------------------------------------------------------------------
// API chamada pelo interpretador (`call_module` / métodos de handle)
// ---------------------------------------------------------------------------

fn handle(id: i64) -> Option<Arc<UiShared>> {
    let guard = windows().lock().ok()?;
    let h = guard.get(&id)?;
    Some(h.ui.clone())
}

fn send(id: i64, cmd: Cmd) {
    let tx = {
        let Ok(guard) = windows().lock() else { return };
        match guard.get(&id) {
            Some(w) => w.tx.clone(),
            None => return,
        }
    };
    let _ = tx.send(cmd);
}

pub fn window_create(title: String, width: f32, height: f32) -> i64 {
    let id = next_window_id();
    let (tx, rx) = std::sync::mpsc::channel::<Cmd>();
    let autoquit = std::env::var("LEXICON_GUI_AUTOQUIT")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok());
    let ui = Arc::new(UiShared::new(width, height, autoquit));
    let title = if title.is_empty() { "Lexicon".to_string() } else { title };
    windows().lock().unwrap().insert(
        id,
        WindowHandle { tx, ui: ui.clone(), title, width, height, spawned: false },
    );
    receivers().lock().unwrap().insert(id, rx);

    start_host_once();
    id
}

/// Sobe o host eframe (UM event loop por processo — winit). Idempotente.
fn start_host_once() {
    static STARTED: AtomicBool = AtomicBool::new(false);
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    // macOS: o event loop EXIGE a main thread (sem escape via winit) e o
    // interpretador já roda em thread própria — janelas ficam indisponíveis
    // aqui (usar `lex gui`). Windows/Linux: `with_any_thread`.
    #[cfg(target_os = "macos")]
    return;

    #[cfg(not(target_os = "macos"))]
    {
        let _ = std::thread::Builder::new()
            .name("lex-gui".into())
            .spawn(|| {
                let mut options = eframe::NativeOptions {
                    // Raiz invisível: as janelas reais são viewports.
                    viewport: egui::ViewportBuilder::default().with_visible(false),
                    renderer: eframe::Renderer::Wgpu,
                    ..Default::default()
                };
                // winit 0.30 no Windows: event loop fora da main thread
                // exige `EventLoopBuilderExtWindows::with_any_thread(true)`.
                #[cfg(windows)]
                {
                    options.event_loop_builder = Some(Box::new(|builder| {
                        use winit::platform::windows::EventLoopBuilderExtWindows as _;
                        builder.with_any_thread(true);
                    }));
                }
                let res = eframe::run_native(
                    "Lexicon",
                    options,
                    Box::new(|_cc| Ok(Box::new(Host::default()))),
                );
                match res {
                    Ok(()) => debug_log("host: event loop saiu (ok)"),
                    Err(e) => debug_log(&format!("host: event loop ERRO: {}", e)),
                }
                // Sem host: marca todas as janelas como fechadas p/ o
                // interpretador nunca travar em `wait_windows`.
                for h in windows().lock().unwrap().values() {
                    let mut st = h.ui.st.lock().unwrap();
                    st.should_close = true;
                    st.closed_done = true;
                    h.ui.cv.notify_all();
                }
            });
    }
}

fn debug_log(msg: &str) {
    if std::env::var("LEXICON_GUI_DEBUG").is_ok() {
        eprintln!("[gfx-debug] {}", msg);
    }
}

pub fn window_should_close(id: i64) -> bool {
    handle(id).map_or(true, |ui| ui.st.lock().unwrap().should_close)
}

/// No-op simétrico à API de jogos; o host egui bombeia sozinho.
pub fn window_poll(_id: i64) {}

/// Espera o host pintar o frame desta janela (pacing do loop de jogo).
pub fn window_present(id: i64) {
    let Some(ui) = handle(id) else { return };
    // Marca a fronteira do quadro: tudo que foi enviado até aqui vira o
    // próximo quadro publicado (veja `Cena` em render_window).
    send(id, Cmd::Present);
    let mut st = ui.st.lock().unwrap();
    loop {
        if st.frame_done || st.closed_done {
            st.frame_done = false;
            return;
        }
        let (next, _) = ui
            .cv
            .wait_timeout(st, Duration::from_millis(200))
            .unwrap();
        st = next;
    }
}

pub fn window_close(id: i64) {
    if let Some(ui) = handle(id) {
        // Deixa de ser declarada → o egui poda a viewport e o eframe
        // destrói a janela no próximo pass. `closed_done` imediato evita
        // que `wait_windows` dependa de mais um frame de render.
        let mut st = ui.st.lock().unwrap();
        st.should_close = true;
        st.closed_done = true;
        drop(st);
        ui.cv.notify_all();
    }
    if let Ok(mut s) = scenes().lock() {
        s.remove(&id);
    }
    send(id, Cmd::Close);
}

pub fn window_set_title(id: i64, t: String) {
    send(id, Cmd::SetTitle(t));
}

pub fn window_set_size(id: i64, w: f32, h: f32) {
    send(id, Cmd::SetSize(w, h));
    if let Some(ui) = handle(id) {
        let mut st = ui.st.lock().unwrap();
        st.width = w;
        st.height = h;
    }
}

pub fn window_width(id: i64) -> f32 {
    handle(id).map_or(0.0, |ui| ui.st.lock().unwrap().width)
}

pub fn window_height(id: i64) -> f32 {
    handle(id).map_or(0.0, |ui| ui.st.lock().unwrap().height)
}

pub fn window_backend(id: i64) -> String {
    let Some(ui) = handle(id) else {
        return "indisponível".to_string();
    };
    let mut st = ui.st.lock().unwrap();
    // O adapter só é conhecido após a primeira frame do host — espera até 2s
    // (o host acorda o condvar ao fim de cada frame).
    if st.backend.is_empty() && !st.closed_done {
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        while st.backend.is_empty() && !st.closed_done {
            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }
            let (next, _) = ui.cv.wait_timeout(st, deadline - now).unwrap();
            st = next;
        }
    }
    if st.backend.is_empty() {
        "iniciando (wgpu)".to_string()
    } else {
        st.backend.clone()
    }
}

fn to_color(c: [f32; 4]) -> egui::Color32 {
    let cl = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    egui::Color32::from_rgba_unmultiplied(cl(c[0]), cl(c[1]), cl(c[2]), cl(c[3]))
}

pub fn canvas_clear(id: i64, c: [f32; 4]) {
    if let Some(ui) = handle(id) {
        ui.st.lock().unwrap().bg = c;
    }
}

pub fn canvas_fill_rect(id: i64, x: f32, y: f32, w: f32, h: f32, c: [f32; 4]) {
    send(id, Cmd::Rect { x, y, w, h, c });
}

pub fn canvas_fill_circle(id: i64, x: f32, y: f32, r: f32, c: [f32; 4]) {
    send(id, Cmd::Circle { x, y, r, c });
}

pub fn canvas_line(id: i64, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, c: [f32; 4]) {
    send(id, Cmd::Line { x1, y1, x2, y2, w, c });
}

pub fn canvas_text(id: i64, x: f32, y: f32, s: String, size: f32, c: [f32; 4]) {
    send(id, Cmd::Text { x, y, s, size, c });
}

// --- Input ------------------------------------------------------------------

/// Normaliza nomes vindos de Lex: `"arrowUp"`/`"up"` → `up`, `"ESC"` → `escape`.
fn norm_key(k: &str) -> String {
    let k = k.trim().to_lowercase();
    let k = k.strip_prefix("arrow").unwrap_or(&k).to_string();
    match k.as_str() {
        "esc" => "escape".into(),
        "spacebar" | " " => "space".into(),
        "return" => "enter".into(),
        "ctrl" | "control" => "control".into(),
        other => other.to_string(),
    }
}

/// Nome canônico (minúsculas) de uma tecla egui: `Key::ArrowUp` → `up`.
pub fn key_name(k: egui::Key) -> String {
    let s = format!("{:?}", k).replace("Arrow", "");
    s.to_lowercase()
}

pub fn input_key_down(id: i64, key: &str) -> bool {
    handle(id)
        .map_or(false, |ui| ui.st.lock().unwrap().keys.contains(&norm_key(key)))
}

pub fn input_mouse_x(id: i64) -> f32 {
    handle(id).map_or(0.0, |ui| ui.st.lock().unwrap().mouse.0)
}

pub fn input_mouse_y(id: i64) -> f32 {
    handle(id).map_or(0.0, |ui| ui.st.lock().unwrap().mouse.1)
}

pub fn input_mouse_down(id: i64) -> bool {
    handle(id).map_or(false, |ui| ui.st.lock().unwrap().mouse_down)
}

/// Consome os cliques pendentes do widget (um frame de vida).
pub fn input_clicked(widget_id: i64) -> bool {
    let mut hit = false;
    for w in windows().lock().unwrap().values() {
        let mut st = w.ui.st.lock().unwrap();
        if let Some(pos) = st.clicked.iter().position(|&i| i == widget_id) {
            st.clicked.remove(pos);
            hit = true;
        }
    }
    hit
}

pub fn input_value(widget_id: i64) -> String {
    widget_text(widget_id).unwrap_or_default()
}

pub fn input_checked(widget_id: i64) -> bool {
    widget_checked(widget_id).unwrap_or(false)
}

// --- Widgets / menus ----------------------------------------------------------

pub fn widget_new(kind: Widget) -> i64 {
    store().lock().unwrap().alloc(kind)
}

// --- Mutação de widgets (métodos `setText`/`setPlaceholder`/...) ----------

/// Texto canônico de um widget de texto: valor digitado (TextField) ou
/// label (Label/Button). `None` = id desconhecido.
pub fn widget_text(id: i64) -> Option<String> {
    let st = store().lock().unwrap();
    match st.widgets.get(&id)? {
        Widget::TextField { .. } => {
            for w in windows().lock().ok()?.values() {
                if let Some(v) = w.ui.st.lock().unwrap().values.get(&id) {
                    return Some(v.clone());
                }
            }
            st.defaults_s.get(&id).cloned()
        }
        Widget::Label { text } | Widget::Button { label: text } => Some(text.clone()),
        _ => None,
    }
}

/// `TextField::setText` / `Label::setText` / `Button::setText`.
pub fn widget_set_text(id: i64, s: &str) -> bool {
    let mut st = store().lock().unwrap();
    match st.widgets.get_mut(&id) {
        Some(Widget::TextField { .. }) => {
            st.defaults_s.insert(id, s.to_string());
            for w in windows().lock().unwrap().values() {
                w.ui.st.lock().unwrap().values.insert(id, s.to_string());
            }
            true
        }
        Some(Widget::Label { text }) => {
            *text = s.to_string();
            true
        }
        Some(Widget::Button { label }) => {
            *label = s.to_string();
            true
        }
        _ => false,
    }
}

pub fn widget_set_placeholder(id: i64, s: &str) -> bool {
    let mut st = store().lock().unwrap();
    match st.widgets.get_mut(&id) {
        Some(Widget::TextField { placeholder }) => {
            *placeholder = s.to_string();
            true
        }
        _ => false,
    }
}

pub fn widget_checked(id: i64) -> Option<bool> {
    let st = store().lock().unwrap();
    match st.widgets.get(&id)? {
        Widget::Checkbox { .. } => {
            for w in windows().lock().ok()?.values() {
                if let Some(&c) = w.ui.st.lock().unwrap().checks.get(&id) {
                    return Some(c);
                }
            }
            st.defaults_b.get(&id).copied()
        }
        _ => None,
    }
}

pub fn widget_set_checked(id: i64, c: bool) -> bool {
    let mut st = store().lock().unwrap();
    match st.widgets.get(&id) {
        Some(Widget::Checkbox { .. }) => {
            st.defaults_b.insert(id, c);
            for w in windows().lock().unwrap().values() {
                w.ui.st.lock().unwrap().checks.insert(id, c);
            }
            true
        }
        _ => false,
    }
}

pub fn widget_add(window_id: i64, widget_id: i64) {
    store()
        .lock()
        .unwrap()
        .window_widgets
        .entry(window_id)
        .or_default()
        .push(widget_id);
}

pub fn menu_add(parent: i64, child: i64) {
    store()
        .lock()
        .unwrap()
        .children
        .entry(parent)
        .or_default()
        .push(child);
}

pub fn window_set_menubar(window_id: i64, bar_id: i64) {
    // O host lê esta árvore direto do Store a cada frame.
    store()
        .lock()
        .unwrap()
        .window_menubar
        .insert(window_id, bar_id);
}

/// Bloqueia até todas as janelas abertas fecharem (apps event-driven:
/// o `main()` pode terminar e a UI continua viva até o usuário fechar).
pub fn wait_windows() {
    let debug = std::env::var("LEXICON_GUI_DEBUG").is_ok();
    let mut ticks: u64 = 0;
    loop {
        let mut alive = 0;
        for w in windows().lock().unwrap().values() {
            if !w.ui.st.lock().unwrap().closed_done {
                alive += 1;
            }
        }
        if alive == 0 {
            return;
        }
        if debug {
            ticks += 1;
            if ticks % 20 == 0 {
                for (id, w) in windows().lock().unwrap().iter() {
                    let st = w.ui.st.lock().unwrap();
                    debug_log(&format!(
                        "win#{} frames={} should_close={} closed={} backend={}",
                        id, st.frames, st.should_close, st.closed_done, st.backend
                    ));
                }
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ---------------------------------------------------------------------------
// Host: a App egui única — cria viewports por janela e as desenha
// ---------------------------------------------------------------------------

#[derive(Default)]
struct Host {
    /// Adapter wgpu (1×, publicado em toda janela que ainda não o tem).
    backend_name: Option<String>,
    /// Diagnóstico: total de passes do root.
    updates: u64,
    /// Janelas já registradas em log (1× por janela).
    logged: HashSet<i64>,
}

impl Host {
    fn backend_name(b: eframe::wgpu::Backend) -> &'static str {
        match b {
            eframe::wgpu::Backend::Vulkan => "Vulkan",
            eframe::wgpu::Backend::Dx12 => "DirectX 12",
            eframe::wgpu::Backend::Metal => "Metal",
            eframe::wgpu::Backend::Gl => "OpenGL",
            eframe::wgpu::Backend::BrowserWebGpu => "WebGPU",
            _ => "desconhecido",
        }
    }

    /// Publica o adapter (1×) em todas as janelas e acorda quem espera
    /// `Window::backend`.
    fn publish_backend(&mut self, frame: &eframe::Frame) {
        if self.backend_name.is_none() {
            if let Some(rs) = frame.wgpu_render_state() {
                let name = Self::backend_name(rs.adapter.get_info().backend).to_string();
                self.backend_name = Some(name);
                debug_log(&format!("host: backend={}", self.backend_name.clone().unwrap()));
            }
        }
        if let Some(name) = &self.backend_name {
            for w in windows().lock().unwrap().values() {
                let mut st = w.ui.st.lock().unwrap();
                if st.backend.is_empty() {
                    st.backend = name.clone();
                    w.ui.cv.notify_all();
                }
            }
        }
    }

    /// DECLARA as janelas vivas como viewports — a cada pass. O egui poda
    /// filhas de um viewport que não forem re-declaradas naquele pass
    /// ("it was never used this pass"), então redeclarar é obrigatório.
    fn declare_viewports(&mut self, ctx: &egui::Context) {
        let targets: Vec<(i64, String, f32, f32, bool)> = {
            let mut all = windows().lock().unwrap();
            let mut out = Vec::new();
            for (id, h) in all.iter_mut() {
                {
                    let st = h.ui.st.lock().unwrap();
                    if st.closed_done || st.should_close {
                        continue;
                    }
                }
                let first = !h.spawned;
                h.spawned = true;
                out.push((*id, h.title.clone(), h.width, h.height, first));
            }
            out
        };
        for (id, title, w, h, first) in targets {
            let vid = egui::ViewportId::from_hash_of(&id);
            let builder = egui::ViewportBuilder::default()
                .with_title(title)
                .with_inner_size([w.max(1.0), h.max(1.0)]);
            if first && self.logged.insert(id) {
                debug_log(&format!("host: declarando viewport win#{}", id));
            }
            // Viewports filhas não repintam sozinhas (só com input/foco) —
            // pede repaint explícito p/ todas as janelas vivas (~60fps).
            ctx.request_repaint_of(vid);
            ctx.show_viewport_deferred(vid, builder, move |vctx, _vid| {
                render_window(vctx, id);
            });
        }
    }
}

impl eframe::App for Host {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.updates += 1;
        if std::env::var("LEXICON_GUI_DEBUG").is_ok()
            && (self.updates <= 3 || self.updates % 60 == 0)
        {
            debug_log(&format!(
                "host update #{} (embed={}, cur_viewport={:?})",
                self.updates,
                ctx.embed_viewports(),
                ctx.viewport_id(),
            ));
        }
        self.publish_backend(frame);
        self.declare_viewports(ctx);
        // Mantém o loop bombeando mesmo sem input (jogos dependem disso;
        // cada viewport repinta junto com a raiz).
        ctx.request_repaint_after(Duration::from_millis(12));
    }
}

/// Desenha UMA janela Lex dentro da sua viewport (1× por frame).
fn render_window(ctx: &egui::Context, id: i64) {
    let dbg = std::env::var("LEXICON_GUI_DEBUG").is_ok();
    let Some(ui) = handle(id) else { return };

    // 1) Drena os comandos de desenho do frame corrente (immediate mode).
    let mut ops: Vec<Cmd> = Vec::new();
    let mut close_cmd = false;
    let mut fronteira = false;
    if let Ok(recv) = receivers().lock() {
        if let Some(rx) = recv.get(&id) {
            while let Ok(cmd) = rx.try_recv() {
                match &cmd {
                    Cmd::SetTitle(t) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(t.clone()))
                    }
                    Cmd::SetSize(w, h) => {
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(*w, *h)));
                        let mut st = ui.st.lock().unwrap();
                        st.width = *w;
                        st.height = *h;
                    }
                    Cmd::Present => fronteira = true,
                    Cmd::Close => close_cmd = true,
                    _ => ops.push(cmd),
                }
            }
        }
    }

    // O host repinta a viewport muito mais rápido do que o interpretador
    // produz quadros, então o quadro só é trocado na fronteira que o próprio
    // programa Lex marca (`Window::present`); entre uma e outra, redesenha-se
    // o último quadro completo — senão cada repintura mostraria um pedaço.
    let cena: Vec<Cmd> = {
        let mut out: Vec<Cmd> = Vec::new();
        if let Ok(mut s) = scenes().lock() {
            let c = s.entry(id).or_default();
            if !ops.is_empty() {
                c.pendente.extend(ops.drain(..));
            }
            if fronteira && !c.pendente.is_empty() {
                c.publicada = std::mem::take(&mut c.pendente);
            }
            out = c.publicada.clone();
        }
        out
    };
    let ops = cena;

    let bg = to_color(ui.st.lock().unwrap().bg);

    // 2) Menus (TopBottomPanel) — a árvore vive no Store global.
    let menubar = store().lock().unwrap().window_menubar.get(&id).copied();
    if menubar.is_some() {
        egui::TopBottomPanel::top(format!("lex_menu_bar_{}", id)).show(ctx, |ui| {
            let mut clicks: Vec<i64> = Vec::new();
            {
                let store = store().lock().unwrap();
                let kids = store
                    .children
                    .get(&menubar.unwrap_or(0))
                    .cloned()
                    .unwrap_or_default();
                ui.horizontal(|ui| {
                    for menu_id in kids {
                        let title = match store.widgets.get(&menu_id) {
                            Some(Widget::Menu { title }) => title.clone(),
                            _ => "Menu".to_string(),
                        };
                        let items = store.children.get(&menu_id).cloned().unwrap_or_default();
                        ui.menu_button(&title, |ui| {
                            for item_id in items {
                                let (label, shortcut) = match store.widgets.get(&item_id) {
                                    Some(Widget::MenuItem { label, shortcut }) => {
                                        (label.clone(), shortcut.clone())
                                    }
                                    _ => continue,
                                };
                                let text = if shortcut.is_empty() {
                                    label
                                } else {
                                    format!("{}\t{}", label, shortcut)
                                };
                                if ui.button(text).clicked() {
                                    clicks.push(item_id);
                                }
                            }
                        });
                    }
                });
            }
            if !clicks.is_empty() {
                if let Some(u) = handle(id) {
                    u.st.lock().unwrap().clicked.extend(clicks);
                }
            }
        });
    }

    // 3) Painel central: widgets empilhados + canvas (ops) por cima.
    let mut clicks: Vec<i64> = Vec::new();
    egui::CentralPanel::default()
        .frame(egui::Frame::NONE.fill(bg))
        .show(ctx, |ui| {
            let widget_ids = store()
                .lock()
                .unwrap()
                .window_widgets
                .get(&id)
                .cloned()
                .unwrap_or_default();

            if !widget_ids.is_empty() {
                ui.vertical(|ui| {
                    let store = store().lock().unwrap();
                    for wid in &widget_ids {
                        match store.widgets.get(wid).cloned() {
                            Some(Widget::Label { text }) => {
                                ui.label(text);
                                ui.add_space(6.0);
                            }
                            Some(Widget::Button { label }) => {
                                if ui.button(label).clicked() {
                                    clicks.push(*wid);
                                }
                                ui.add_space(6.0);
                            }
                            Some(Widget::TextField { placeholder }) => {
                                let win_ui = handle(id);
                                let mut v = {
                                    let mut v = store
                                        .defaults_s
                                        .get(wid)
                                        .cloned()
                                        .unwrap_or_default();
                                    if let Some(u) = &win_ui {
                                        if let Some(user) =
                                            u.st.lock().unwrap().values.get(wid)
                                        {
                                            v = user.clone();
                                        }
                                    }
                                    v
                                };
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut v)
                                        .hint_text(&placeholder)
                                        .desired_width(240.0),
                                );
                                if resp.changed() {
                                    if let Some(u) = &win_ui {
                                        u.st.lock().unwrap().values.insert(*wid, v);
                                    }
                                }
                                ui.add_space(6.0);
                            }
                            Some(Widget::Checkbox { label }) => {
                                let win_ui = handle(id);
                                let mut c = {
                                    let mut c =
                                        store.defaults_b.get(wid).copied().unwrap_or(false);
                                    if let Some(u) = &win_ui {
                                        if let Some(&user) =
                                            u.st.lock().unwrap().checks.get(wid)
                                        {
                                            c = user;
                                        }
                                    }
                                    c
                                };
                                if ui.checkbox(&mut c, label).changed() {
                                    if let Some(u) = &win_ui {
                                        u.st.lock().unwrap().checks.insert(*wid, c);
                                    }
                                }
                                ui.add_space(6.0);
                            }
                            _ => {}
                        }
                    }
                });
                ui.separator();
            }

            // Canvas: (0,0) no canto superior esquerdo da área útil.
            let (resp, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::hover());
            for op in &ops {
                match op {
                    Cmd::Rect { x, y, w, h, c } => {
                        painter.rect_filled(
                            egui::Rect::from_min_size(
                                egui::pos2(*x, *y),
                                egui::vec2((*w).max(0.0), (*h).max(0.0)),
                            ),
                            0.0,
                            to_color(*c),
                        );
                    }
                    Cmd::Circle { x, y, r, c } => {
                        painter.circle_filled(egui::pos2(*x, *y), (*r).max(0.0), to_color(*c));
                    }
                    Cmd::Line { x1, y1, x2, y2, w, c } => {
                        painter.line_segment(
                            [egui::pos2(*x1, *y1), egui::pos2(*x2, *y2)],
                            egui::Stroke::new((*w).max(0.5), to_color(*c)),
                        );
                    }
                    Cmd::Text { x, y, s, size, c } => {
                        painter.text(
                            egui::pos2(*x, *y),
                            egui::Align2::LEFT_TOP,
                            s,
                            egui::FontId::proportional((*size).max(4.0)),
                            to_color(*c),
                        );
                    }
                    _ => {}
                }
            }
            // Mouse em coordenadas locais do canvas.
            if let Some(u) = handle(id) {
                let mut st = u.st.lock().unwrap();
                if let Some(p) = ctx.input(|i| i.pointer.latest_pos()) {
                    st.mouse = (p.x - resp.rect.min.x, p.y - resp.rect.min.y);
                }
                st.mouse_down = ctx.input(|i| i.pointer.primary_down());
            }
        });

    // 4) Snapshot de entrada + fechamento + pacing.
    let mut keys: HashSet<String> = HashSet::new();
    ctx.input(|i| {
        for k in &i.keys_down {
            keys.insert(key_name(*k));
        }
    });
    let close_req = ctx.input(|i| i.viewport().close_requested());

    {
        let mut st = ui.st.lock().unwrap();
        st.keys = keys;
        if !clicks.is_empty() {
            st.clicked.extend(clicks);
        }
        if close_req {
            st.should_close = true;
            st.closed_done = true;
        }
        st.frames += 1;
        if let Some(n) = st.autoquit {
            if st.frames >= n {
                st.should_close = true;
            }
        }
        let do_close = (st.should_close || close_cmd) && !st.close_sent;
        if do_close {
            st.close_sent = true;
            // Foi ordenado o fechamento (autoquit/`Window::close`): esta é a
            // última frame desta janela — marca fechada e para de declarar.
            st.closed_done = true;
        }
        st.frame_done = true;
        ui.cv.notify_all();
        if do_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    if dbg {
        debug_log(&format!("win#{} frame pintado", id));
    }
}
