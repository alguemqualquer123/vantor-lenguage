// e2e_148 - RGB Triangle (o "primeiro triângulo" do OpenGL) como teste de
// rasterizador: winding/backface culling, pesos baricêntricos, cores Gouraud,
// mapeamento NDC -> janela (glViewport) e cobertura de pixels num framebuffer.
module e2e_148;

// Os três vértices clássicos, em coordenadas normalizadas (x para a direita,
// y para cima): vermelho embaixo à esquerda, verde embaixo à direita,
// azul no topo.
pub const V0X = -0.5;
pub const V0Y = -0.5;
pub const V1X = 0.5;
pub const V1Y = -0.5;
pub const V2X = 0.0;
pub const V2Y = 0.5;

// dupla área signed do triângulo acima: (1.0 * 1.0) - (0.0 * 0.5) = 1.0
pub const DEN = 1.0;
pub const TOL = 0.000000001;

// framebuffer 16x16 amostrado no centro de cada pixel (2.0 / 16)
pub const LARG = 16;
pub const ALT = 16;
pub const PIX = 256;
pub const PASSO = 0.125;

// Saída do baricêntrico: os pesos SÃO a cor do fragmento (interpolação
// Gouraud de vertex colors vermelho/verde/azul).
let wr = 0.0;
let wg = 0.0;
let wb = 0.0;

// Saída de para_janela: posição do vértice em pixels da tela.
let jx = 0.0;
let jy = 0.0;

let fb_r = [];
let fb_g = [];
let fb_b = [];
let fb_a = [];

fn area2(ax: float, ay: float, bx: float, by: float, cx: float, cy: float) -> float {
    return (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
}

/// Pesos baricêntricos de (px,py) em wr/wg/wb; true quando o ponto cai
/// dentro do triângulo (todos os pesos não negativos).
fn baricentrico(px: float, py: float) -> bool {
    wr = area2(px, py, V1X, V1Y, V2X, V2Y) / DEN;
    wg = area2(px, py, V2X, V2Y, V0X, V0Y) / DEN;
    wb = area2(px, py, V0X, V0Y, V1X, V1Y) / DEN;
    return wr + TOL >= 0.0 && wg + TOL >= 0.0 && wb + TOL >= 0.0;
}

/// Como glViewport(0, 0, larg, alt): [-1,1] -> pixels, com o eixo y virado
/// porque a origem da tela fica no canto superior esquerdo.
fn para_janela(nx: float, ny: float, larg: float, alt: float) -> void {
    jx = (nx * 0.5 + 0.5) * larg;
    jy = (1.0 - (ny * 0.5 + 0.5)) * alt;
}

fn perto(a: float, b: float) -> bool {
    return Math::abs(a - b) < TOL;
}

fn pixels_da_linha(l: i64) -> i64 {
    let n = 0;
    let x = 0;
    while x < LARG {
        if fb_a[l * LARG + x] > 0.5 {
            n = n + 1;
        }
        x = x + 1;
    }
    return n;
}

pub fn main() -> void {
    // 1) Winding: anti-horário é a face da frente; horário é descartada.
    let face = area2(V0X, V0Y, V1X, V1Y, V2X, V2Y);
    assert(face == DEN, "dupla área do triângulo RGB é 1.0");
    assert(face > 0.0, "sentido anti-horário = face da frente");
    assert(area2(V2X, V2Y, V1X, V1Y, V0X, V0Y) < 0.0, "sentido horário = culled");

    // 2) Cor de cada vértice = seu peso baricêntrico.
    assert(baricentrico(V0X, V0Y), "v0 é coberto");
    assert(wr == 1.0 && wg == 0.0 && wb == 0.0, "v0 -> RGB(1,0,0) vermelho");
    assert(baricentrico(V1X, V1Y), "v1 é coberto");
    assert(wr == 0.0 && wg == 1.0 && wb == 0.0, "v1 -> RGB(0,1,0) verde");
    assert(baricentrico(V2X, V2Y), "v2 é coberto");
    assert(wr == 0.0 && wg == 0.0 && wb == 1.0, "v2 -> RGB(0,0,1) azul");

    // 3) Partição da unidade: r + g + b == 1 em qualquer fragmento.
    assert(baricentrico(0.0, 0.0), "origem é coberta");
    assert(wr == 0.25 && wg == 0.25 && wb == 0.5, "origem -> RGB(0.25,0.25,0.5)");
    assert(wr + wg + wb == 1.0, "pesos somam 1 na origem");

    // 4) Interpolação linear nas arestas.
    assert(baricentrico(-0.25, 0.0), "meio de v0->v2 é coberto");
    assert(wr == 0.5 && wg == 0.0 && wb == 0.5, "meio da aresta esquerda é magenta");
    assert(baricentrico(0.0, -0.5), "meio da base é coberto");
    assert(wr == 0.5 && wg == 0.5 && wb == 0.0, "meio da base é amarelo (azul zerado)");

    // 5) Fragmentos fora do triângulo não são cobertos.
    assert(!baricentrico(-0.9, -0.9), "canto inferior esquerdo fora");
    assert(!baricentrico(0.9, -0.9), "canto inferior direito fora");
    assert(!baricentrico(0.0, 0.9), "acima do ápice fora");
    assert(!baricentrico(0.75, 0.0), "direita do ápice fora");

    // 6) Gradiente RGB: vermelho cresce para -x, verde para +x, simétrico.
    assert(baricentrico(-0.2, -0.2), "ponto à esquerda é coberto");
    let r_esq = wr;
    let g_esq = wg;
    let b_esq = wb;
    assert(baricentrico(0.2, -0.2), "ponto à direita é coberto");
    assert(r_esq > wr, "o vermelho decresce ao longo de +x");
    assert(g_esq < wg, "o verde cresce ao longo de +x");
    assert(perto(r_esq, wg), "espelho x troca vermelho e verde");
    assert(perto(g_esq, wr), "espelho x troca verde e vermelho");
    assert(perto(b_esq, wb), "o azul é igual nas duas amostras espelhadas");

    // 7) glViewport: NDC -> pixels da janela, y virado.
    para_janela(V0X, V0Y, 800.0, 600.0);
    assert(jx == 200.0 && jy == 450.0, "v0 -> pixel (200,450)");
    para_janela(V1X, V1Y, 800.0, 600.0);
    assert(jx == 600.0 && jy == 450.0, "v1 -> pixel (600,450)");
    para_janela(V2X, V2Y, 800.0, 600.0);
    assert(jx == 400.0 && jy == 150.0, "v2 -> pixel (400,150)");
    assert(jy < 450.0, "o ápice azul fica ACIMA da base na tela");

    // 8) Rasterização: preenche o framebuffer 16x16 no centro de cada pixel.
    let i = 0;
    while i < PIX {
        fb_r.push(0.0);
        fb_g.push(0.0);
        fb_b.push(0.0);
        fb_a.push(0.0);
        i = i + 1;
    }

    let cobertos = 0;
    let acima_vazio = 0;
    let soma_cores = 0.0;
    let y = 0;
    while y < ALT {
        let x = 0;
        while x < LARG {
            let nx = ((x as float) + 0.5) * PASSO - 1.0;
            let ny = 1.0 - ((y as float) + 0.5) * PASSO;
            let idx = y * LARG + x;
            if baricentrico(nx, ny) {
                fb_r[idx] = wr;
                fb_g[idx] = wg;
                fb_b[idx] = wb;
                fb_a[idx] = 1.0;
                cobertos = cobertos + 1;
                soma_cores = soma_cores + wr + wg + wb;
                if idx < 80 {
                    acima_vazio = acima_vazio + 1;
                }
            }
            x = x + 1;
        }
        y = y + 1;
    }

    // A área do triângulo é 0.5 de um quadrado de área 4 -> 1/8 dos pixels.
    assert(cobertos == 32, "32 de 256 pixels cobertos (12.5%)");
    assert(perto(soma_cores, (cobertos as float)), "r+g+b=1 em todos os fragmentos");
    assert(acima_vazio == 0, "as 5 primeiras linhas ficam vazias (acima do ápice)");

    // Valores de referência batidos com uma implementação independente.
    assert(fb_a[86] == 0.0, "pixel 86 fica fora do triângulo");
    assert(fb_r[87] == 0.15625, "pixel 87: vermelho 5/32");
    assert(fb_g[87] == 0.03125, "pixel 87: verde 1/32");
    assert(fb_b[87] == 0.8125, "pixel 87: azul 26/32");
    assert(fb_r[88] == 0.03125 && fb_g[88] == 0.15625, "pixel 88 espelha o 87");
    assert(fb_a[187] == 1.0, "pixel 187 é o último coberto da varredura");
    assert(fb_r[187] == 0.03125 && fb_b[187] == 0.0625, "pixel 187: quase verde");

    // Scanline: o triângulo alarga 2 pixels por linha, da base ao ápice.
    assert(pixels_da_linha(5) == 2, "linha 5 tem 2 pixels");
    assert(pixels_da_linha(7) == 4, "linha 7 tem 4 pixels");
    assert(pixels_da_linha(9) == 6, "linha 9 tem 6 pixels");
    assert(pixels_da_linha(11) == 8, "linha 11 tem 8 pixels");
    assert(pixels_da_linha(12) == 0, "linha 12 (abaixo da base) fica vazia");
    return;
}
