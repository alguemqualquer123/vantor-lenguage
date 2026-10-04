// Lexicon SDK — native signature stubs (Input — teclado/mouse/widgets).
// Documentation for IDE navigation ONLY (see native/console.lex header).

/// Tecla pressionada neste frame? (`"up"`, `"down"`, `"space"`, `"w"`, ...)
pub fn keyDown(win: Window, key: String) -> bool {
    panic("native stub");
}

/// X do mouse em coordenadas locais do canvas.
pub fn mouseX(win: Window) -> float {
    panic("native stub");
}

/// Y do mouse em coordenadas locais do canvas.
pub fn mouseY(win: Window) -> float {
    panic("native stub");
}

/// Botão principal do mouse pressionado?
pub fn mouseDown(win: Window) -> bool {
    panic("native stub");
}

/// O widget (Button/MenuItem) foi clicado? Consome o clique.
pub fn clicked(widget: Widget) -> bool {
    panic("native stub");
}

/// Texto atual de um TextField.
pub fn value(widget: Widget) -> String {
    panic("native stub");
}

/// Estado de um Checkbox.
pub fn checked(widget: Widget) -> bool {
    panic("native stub");
}
