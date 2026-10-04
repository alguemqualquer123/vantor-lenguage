pub struct Window {
    title: String,
    width: u32,
    height: u32,
}

impl Window {
    pub fn new() -> Self {
        Self {
            title: "Lexicon App".to_string(),
            width: 800,
            height: 600,
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }
}

impl Default for Window {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Part X / §61: native GUI + game stubs (Spec §61).
//
// Contract: no game-engine runtime is required; these are portable
// descriptors + headless logic so games/GUIs build against one shape
// and bind real backends (Vulkan/DX/GL/Metal/WASM-audio/input) per
// platform. All logic here is deterministic and backend-free; platform
// code lives behind the `RenderBackend`/`AudioBackend` selection.
// ---------------------------------------------------------------------------

/// Rendering backend selector (selection only — no context creation).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    Software,
    Vulkan,
    DirectX,
    OpenGl,
    Metal,
    Wgpu,
}

/// Portable GUI event.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Close,
    Resize { width: u32, height: u32 },
    Key { code: u32, pressed: bool },
    Mouse { x: i32, y: i32, pressed: bool },
    Tick { dt_ms: u64 },
}

/// Headless event queue (deterministic; real backends push here).
#[derive(Debug, Default)]
pub struct EventQueue {
    events: std::collections::VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        EventQueue { events: std::collections::VecDeque::new() }
    }

    pub fn push(&mut self, e: Event) {
        self.events.push_back(e);
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Push-button stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Button {
    pub label: String,
    pub pressed: bool,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Button { label: label.into(), pressed: false }
    }

    /// Feed an event; returns `true` on activation (key/mouse press).
    pub fn handle(&mut self, e: &Event) -> bool {
        match e {
            Event::Key { pressed: true, .. } | Event::Mouse { pressed: true, .. } => {
                self.pressed = true;
                true
            }
            _ => false,
        }
    }
}

/// Text label stub.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new(text: impl Into<String>) -> Self {
        Label { text: text.into() }
    }

    pub fn set(&mut self, text: impl Into<String>) {
        self.text = text.into();
    }
}

/// 2-D canvas stub (pixel buffer owned here; GPU upload is a binding).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pixels: Vec<u8>,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let (w, h) = (width.max(1), height.max(1));
        Canvas { width: w, height: h, pixels: vec![0u8; (w as usize) * (h as usize) * 4] }
    }

    pub fn clear(&mut self, rgba: [u8; 4]) {
        for px in self.pixels.chunks_exact_mut(4) {
            px.copy_from_slice(&rgba);
        }
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
}

/// Keyboard/mouse snapshot.
#[derive(Debug, Clone, Default)]
pub struct InputState {
    pub keys: Vec<u32>,
    pub mouse_x: i32,
    pub mouse_y: i32,
    pub mouse_down: bool,
}

impl InputState {
    pub fn new() -> Self {
        InputState::default()
    }

    pub fn key_down(&mut self, code: u32) {
        if !self.keys.contains(&code) {
            self.keys.push(code);
        }
    }

    pub fn key_up(&mut self, code: u32) {
        self.keys.retain(|k| *k != code);
    }

    pub fn is_pressed(&self, code: u32) -> bool {
        self.keys.contains(&code)
    }
}

/// Silent audio stub (volume + play/stop state; real mixing per platform).
#[derive(Debug, Clone)]
pub struct AudioStub {
    pub volume: f32,
    pub playing: bool,
}

impl AudioStub {
    pub fn new(volume: f32) -> Self {
        AudioStub { volume: volume.clamp(0.0, 1.0), playing: false }
    }

    pub fn play(&mut self) {
        self.playing = true;
    }

    pub fn stop(&mut self) {
        self.playing = false;
    }
}

/// Axis-aligned sprite for 2-D game stubs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Sprite {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Sprite { x, y, w, h }
    }

    pub fn overlaps(&self, other: &Sprite) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
    }
}

/// Headless scene: sprite list + fixed-step update (deterministic).
#[derive(Debug, Default)]
pub struct GameScene {
    pub sprites: Vec<Sprite>,
}

impl GameScene {
    pub fn new() -> Self {
        GameScene { sprites: Vec::new() }
    }

    pub fn spawn(&mut self, s: Sprite) -> usize {
        self.sprites.push(s);
        self.sprites.len() - 1
    }

    pub fn step(&mut self, dt: f32, velocity: (f32, f32)) {
        for s in &mut self.sprites {
            s.translate(velocity.0 * dt, velocity.1 * dt);
        }
    }

    pub fn collisions(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for i in 0..self.sprites.len() {
            for j in (i + 1)..self.sprites.len() {
                if self.sprites[i].overlaps(&self.sprites[j]) {
                    out.push((i, j));
                }
            }
        }
        out
    }
}

/// Fixed-step game loop counter (headless; rendering is a binding).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameLoop {
    pub tick: u64,
    pub tick_ms: u64,
}

impl GameLoop {
    pub fn new(tick_ms: u64) -> Self {
        GameLoop { tick: 0, tick_ms: tick_ms.max(1) }
    }

    pub fn step(&mut self) -> u64 {
        self.tick += 1;
        self.tick
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_defaults() {
        let w = Window::new().with_title("Hi").with_size(640, 480);
        assert_eq!(w.title(), "Hi");
        assert_eq!((w.width(), w.height()), (640, 480));
    }

    #[test]
    fn event_queue_fifo() {
        let mut q = EventQueue::new();
        assert!(q.is_empty());
        q.push(Event::Tick { dt_ms: 16 });
        q.push(Event::Close);
        assert_eq!(q.len(), 2);
        assert_eq!(q.pop(), Some(Event::Tick { dt_ms: 16 }));
        assert_eq!(q.pop(), Some(Event::Close));
        assert_eq!(q.pop(), None);
    }

    #[test]
    fn button_activates_on_press() {
        let mut b = Button::new("ok");
        assert!(!b.handle(&Event::Tick { dt_ms: 16 }));
        assert!(b.handle(&Event::Key { code: 13, pressed: true }));
        assert!(b.pressed);
        let mut l = Label::new("a");
        l.set("b");
        assert_eq!(l.text, "b");
    }

    #[test]
    fn canvas_input_audio() {
        let mut c = Canvas::new(2, 2);
        c.clear([255, 0, 0, 255]);
        assert_eq!(c.pixels().len(), 16);
        let mut input = InputState::new();
        input.key_down(65);
        assert!(input.is_pressed(65));
        input.key_up(65);
        assert!(!input.is_pressed(65));
        let mut audio = AudioStub::new(0.5);
        audio.play();
        assert!(audio.playing);
        audio.stop();
        assert!(!audio.playing);
    }

    #[test]
    fn scene_collisions_and_loop() {
        let mut scene = GameScene::new();
        scene.spawn(Sprite::new(0.0, 0.0, 10.0, 10.0));
        scene.spawn(Sprite::new(5.0, 5.0, 10.0, 10.0));
        scene.spawn(Sprite::new(100.0, 100.0, 5.0, 5.0));
        assert_eq!(scene.collisions(), vec![(0, 1)]);
        // Fixed-step translation preserves relative layout (deterministic).
        scene.step(1.0, (10.0, 0.0));
        assert_eq!(scene.collisions(), vec![(0, 1)]);
        scene.sprites[1].translate(1000.0, 0.0);
        assert!(scene.collisions().is_empty());
        let mut game_loop = GameLoop::new(16);
        assert_eq!(game_loop.step(), 1);
        assert_eq!(game_loop.step(), 2);
    }
}
