// Lexicon SDK — native signature stubs (Window — motor gráfico wgpu).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Cria uma janela real (Vulkan/DirectX 12/OpenGL, escolhido pela máquina).
/// `cfg` aceita `title: String`, `width: int`, `height: int`.
pub fn create(cfg: WinConfig) -> Window {
    panic("native stub");
}

/// O usuário pediu fechamento (ou o frame limite de CI foi atingido)?
pub fn shouldClose(win: Window) -> bool {
    panic("native stub");
}

/// Espera a UI pintar o frame atual (pacing do loop de jogo).
pub fn present(win: Window) -> void {
    panic("native stub");
}

/// Bombeia eventos do event loop (no-op: egui roda em sua própria thread).
pub fn poll(win: Window) -> void {
    panic("native stub");
}

/// Fecha a janela.
pub fn close(win: Window) -> void {
    panic("native stub");
}

/// Título da janela.
pub fn setTitle(win: Window, title: String) -> void {
    panic("native stub");
}

/// Redimensiona a janela.
pub fn setSize(win: Window, width: int, height: int) -> void {
    panic("native stub");
}

/// Backend gráfico real usado na máquina (`Vulkan`, `DirectX 12`, `OpenGL`, `Metal`).
pub fn backend(win: Window) -> String {
    panic("native stub");
}

/// Largura atual (px).
pub fn width(win: Window) -> int {
    panic("native stub");
}

/// Altura atual (px).
pub fn height(win: Window) -> int {
    panic("native stub");
}

/// Mostra/foca a janela (no-op: já nasce visível).
pub fn show(win: Window) -> void {
    panic("native stub");
}
