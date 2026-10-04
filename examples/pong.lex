// Pong — demo do motor gráfico do Lex (Window/Canvas/Input reais, wgpu).
// Rodar:  lex run examples/pong.lex
// CI:     lex run --ci examples/pong.lex   (autoquit após ~1s)
//
// O backend (Vulkan / DirectX 12 / OpenGL) depende da máquina:
// imprima `Window::backend(win)` para ver qual foi usado.

pub fn main() -> void {
    let win = Window::create(Title { title: "Pong — Lex", width: 640, height: 400 });

    // Estado do jogo (mutável, sem structs literais).
    let mut pad_h = 70.0;
    let mut py = 165.0; // raquete do jogador (esquerda)
    let mut ay = 165.0; // raquete da CPU (direita)
    let mut bx = 320.0;
    let mut by = 200.0;
    let mut vx = 4.2;
    let mut vy = 2.4;
    let mut ps = 0;
    let mut as_ = 0;
    let mut ticks = 0;

    while !Window::shouldClose(win) {
        // --- input ---
        if Input::keyDown(win, "up") {
            py = py - 6.0;
        }
        if Input::keyDown(win, "down") {
            py = py + 6.0;
        }
        if py < 0.0 {
            py = 0.0;
        }
        if py > 400.0 - pad_h {
            py = 400.0 - pad_h;
        }

        // --- CPU ---
        let target = by - pad_h / 2.0 + 20.0;
        if ay < target - 8.0 {
            ay = ay + 3.4;
        }
        if ay > target + 8.0 {
            ay = ay - 3.4;
        }
        if ay < 0.0 {
            ay = 0.0;
        }
        if ay > 400.0 - pad_h {
            ay = 400.0 - pad_h;
        }

        // --- bola ---
        bx = bx + vx;
        by = by + vy;
        if by < 8.0 {
            by = 8.0;
            vy = 0.0 - vy;
        }
        if by > 392.0 {
            by = 392.0;
            vy = 0.0 - vy;
        }
        // colisão com o jogador
        if bx < 34.0 && bx > 18.0 && by > py - 8.0 && by < py + pad_h + 8.0 {
            vx = 0.0 - vx * 1.03;
            bx = 34.0;
        }
        // colisão com a CPU
        if bx > 606.0 && bx < 622.0 && by > ay - 8.0 && by < ay + pad_h + 8.0 {
            vx = 0.0 - vx * 1.03;
            bx = 606.0;
        }
        // pontos
        if bx < 0.0 {
            as_ = as_ + 1;
            bx = 320.0;
            by = 200.0;
            vx = 4.2;
        }
        if bx > 640.0 {
            ps = ps + 1;
            bx = 320.0;
            by = 200.0;
            vx = 0.0 - 4.2;
        }

        // --- desenho ---
        Canvas::clear(win, 0.06, 0.07, 0.12, 1.0);
        Canvas::line(win, 320.0, 0.0, 320.0, 400.0, 2.0, 0.25, 0.27, 0.35, 1.0);
        Canvas::fillRect(win, 18.0, py, 14.0, pad_h, 0.35, 0.8, 1.0, 1.0);
        Canvas::fillRect(win, 608.0, ay, 14.0, pad_h, 1.0, 0.45, 0.4, 1.0);
        Canvas::fillCircle(win, bx, by, 7.0, 1.0, 1.0, 1.0, 1.0);
        Canvas::text(win, 268.0, 14.0, Text { s: ps + "  " + as_, size: 26.0 }, 1.0, 1.0, 1.0, 1.0);

        // --- pacing (desbloqueia o próximo frame) ---
        Window::present(win);

        ticks = ticks + 1;
        if ticks > 120 && bx < 0.0 {
            break;
        }
    }

    Window::close(win);
}
