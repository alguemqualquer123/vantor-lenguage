// =====================================================
// RGB TRIANGLE — graphics test escrito em Lex
// =====================================================
// O clássico "primeiro triângulo" do OpenGL, mas rasterizado em Lex:
// cada faixa horizontal amostra o centro do pixel, calcula os pesos
// baricêntricos do triângulo e usa esses pesos como cor RGB (Gouraud
// shading com vertex colors vermelho / verde / azul).
//
// A entrega na tela é do motor gráfico nativo da linguagem (wgpu):
// Vulkan, DirectX 12, OpenGL ou Metal, conforme a máquina — o nome real
// aparece no HUD via `Window::backend(win)`.
//
// Rodar:   lex run tests/gfx_rgb_triangle.lex
// Sair:    ESC
//
// Este arquivo NÃO casa no glob tests/e2e_*.lex, então o harness
// (tests/run_e2e.sh) não o executa — é o teste gráfico com janela.
// A versão headless das mesmas contas está em tests/e2e_148.lex.
// =====================================================

struct Title {
    title: String;
    width: int;
    height: int;
}

// ---------- triângulo em coordenadas normalizadas (y para cima) ----------
pub const V0X = -0.5;
pub const V0Y = -0.5;
pub const V1X = 0.5;
pub const V1Y = -0.5;
pub const V2X = 0.0;
pub const V2Y = 0.5;

// dupla área signed do triângulo acima = 1.0
pub const DEN = 1.0;

// resolução da varredura (px)
pub const FAIXA = 4.0;
pub const SEG = 8.0;
pub const TOL = 0.000000001;

// ---------- estado ----------
let win = 0;
let gpu = "wgpu";
let ox = 0.0;
let oy = 0.0;
let lado = 0.0;

let wr = 0.0;
let wg = 0.0;
let wb = 0.0;

let amostras = 0;
let pintados = 0;
let quadros = 0;
let largura = 0.0;
let altura = 0.0;

fn area2(ax: float, ay: float, bx: float, by: float, cx: float, cy: float) -> float {
    return (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
}

/// Pesos baricêntricos de (px,py); true quando cai dentro do triângulo.
fn baricentrico(px: float, py: float) -> bool {
    wr = area2(px, py, V1X, V1Y, V2X, V2Y) / DEN;
    wg = area2(px, py, V2X, V2Y, V0X, V0Y) / DEN;
    wb = area2(px, py, V0X, V0Y, V1X, V1Y) / DEN;
    return wr + TOL >= 0.0 && wg + TOL >= 0.0 && wb + TOL >= 0.0;
}

/// glViewport(0, 0, lado, lado): NDC -> pixels, y virado (origem no topo).
fn px_x(nx: float) -> float {
    return ox + (nx * 0.5 + 0.5) * lado;
}

fn px_y(ny: float) -> float {
    return oy + (1.0 - (ny * 0.5 + 0.5)) * lado;
}

fn definir_viewport() -> void {
    let w = Window::width(win);
    let h = Window::height(win);
    largura = w as float;
    altura = h as float;
    let menor = largura;
    if altura < menor {
        menor = altura;
    }
    lado = menor - 110.0;
    ox = (largura - lado) * 0.5;
    oy = (altura - lado) * 0.5 + 18.0;
}

/// Rasteriza o triângulo em faixas horizontais, cor = baricêntrico.
fn triangular() -> void {
    amostras = 0;
    pintados = 0;
    let faixa = 0.0;
    while faixa < lado {
        let ny = 1.0 - (faixa + FAIXA * 0.5) / lado * 2.0;
        let col_x = 0.0;
        while col_x < lado {
            let nx = (col_x + SEG * 0.5) / lado * 2.0 - 1.0;
            amostras = amostras + 1;
            if baricentrico(nx, ny) {
                Canvas::fill_rect(win, ox + col_x, oy + faixa, SEG, FAIXA, wr, wg, wb, 1.0);
                pintados = pintados + 1;
            }
            col_x = col_x + SEG;
        }
        faixa = faixa + FAIXA;
    }
}

fn desenhar() -> void {
    Canvas::clear(win, 0.02, 0.02, 0.05, 1.0);

    // HUD
    Canvas::text(win, 18, 16, "RGB TRIANGLE - triangle test em Lex", 20, 0.92, 0.94, 1.0, 1.0);
    Canvas::text(win, 18, 42, "GPU: " + gpu, 15, 0.55, 0.85, 1.0, 1.0);

    // moldura e eixos do viewport NDC [-1,1]
    Canvas::line(win, ox, oy, ox + lado, oy, 1.0, 0.28, 0.32, 0.48, 1.0);
    Canvas::line(win, ox + lado, oy, ox + lado, oy + lado, 1.0, 0.28, 0.32, 0.48, 1.0);
    Canvas::line(win, ox + lado, oy + lado, ox, oy + lado, 1.0, 0.28, 0.32, 0.48, 1.0);
    Canvas::line(win, ox, oy + lado, ox, oy, 1.0, 0.28, 0.32, 0.48, 1.0);
    Canvas::line(win, px_x(0.0), oy, px_x(0.0), oy + lado, 1.0, 0.14, 0.16, 0.26, 1.0);
    Canvas::line(win, ox, px_y(0.0), ox + lado, px_y(0.0), 1.0, 0.14, 0.16, 0.26, 1.0);

    // o triângulo em si
    triangular();

    // wireframe por cima das faixas
    Canvas::line(win, px_x(V0X), px_y(V0Y), px_x(V1X), px_y(V1Y), 2.0, 1.0, 1.0, 1.0, 0.9);
    Canvas::line(win, px_x(V1X), px_y(V1Y), px_x(V2X), px_y(V2Y), 2.0, 1.0, 1.0, 1.0, 0.9);
    Canvas::line(win, px_x(V2X), px_y(V2Y), px_x(V0X), px_y(V0Y), 2.0, 1.0, 1.0, 1.0, 0.9);

    // vértices e rótulos
    Canvas::fill_circle(win, px_x(V0X), px_y(V0Y), 5.0, 1.0, 0.15, 0.15, 1.0);
    Canvas::fill_circle(win, px_x(V1X), px_y(V1Y), 5.0, 0.15, 1.0, 0.15, 1.0);
    Canvas::fill_circle(win, px_x(V2X), px_y(V2Y), 5.0, 0.2, 0.4, 1.0, 1.0);
    Canvas::text(win, px_x(V0X) - 26.0, px_y(V0Y) - 6.0, "R", 20, 1.0, 0.35, 0.35, 1.0);
    Canvas::text(win, px_x(V1X) + 10.0, px_y(V1Y) - 6.0, "G", 20, 0.35, 1.0, 0.45, 1.0);
    Canvas::text(win, px_x(V2X) - 6.0, px_y(V2Y) - 26.0, "B", 20, 0.45, 0.65, 1.0, 1.0);

    // auto-verificação impressa: cobertura esperada é 1/8 da área do viewport
    let mil = 0.0;
    if amostras > 0 {
        mil = Math::round((pintados as float) * 1000.0 / (amostras as float));
    }
    let rodape = "amostrados: " + amostras.to_string() + "  |  pintados: " + pintados.to_string() + "  |  cobertura: " + mil.to_string() + " por mil (area terica = 125)  |  ESC para sair";
    Canvas::text(win, 18, altura - 26.0, rodape, 15, 0.8, 0.82, 0.9, 1.0);
}

pub fn main() -> void {
    win = Window::create(Title { title: "RGB Triangle - Lex", width: 760, height: 520 });
    definir_viewport();

    // O quadro é desenhado uma única vez e fica retido na janela (a fronteira
    // de frame é `Window::present`); o loop só segura a janela aberta.
    desenhar();
    Window::present(win);

    while !Window::shouldClose(win) {
        quadros = quadros + 1;
        if Input::keyDown(win, "escape") {
            break;
        }
        if gpu == "wgpu" && quadros > 3 {
            gpu = Window::backend(win);
            definir_viewport();
            desenhar();
        }
        Window::present(win);
    }

    Window::close(win);
    let resultado = "RGB triangle: " + amostras.to_string() + " amostras, " + pintados.to_string() + " fragmentos pintados, GPU: " + gpu;
    Console::log(resultado);
}
