// =====================================================
// COLETOR 3D — jogo 3D em primeira pessoa escrito em Lex
// =====================================================
// A geometria 3D é calculada em Lex: os pontos do mundo são
// transformados para o espaço da câmera (rotação de yaw + pitch),
// projetados em perspectiva e recortados no plano near. O motor
// gráfico nativo da linguagem (Window/Canvas/Input sobre wgpu) entrega
// as linhas na tela — ele escolhe Vulkan, DirectX 12 ou OpenGL conforme
// a máquina; `Window::backend(win)` informa qual foi usado.
//
// Rodar:  lex run              (dentro de apps/jogo-3d)
// Controles: W A S D mover · SHIFT correr · ESPAÇO pular
//            setas ou arrastar o mouse = virar a câmera · ESC sair
// Meta: coletar cristais. Cada cristal devolve 3,5 s de relógio.

struct Title {
    title: String;
    width: int;
    height: int;
}

// ---------- configuração ----------
pub const LIMITE = 40.0;
pub const FRAMES_MAX = 4000;
pub const MEIO = 26.0;
pub const NEAR = 0.25;
pub const OLHO = 1.7;
pub const PASSO = 8.0;
pub const NEOX = 3;
pub const EXT = 20.0;
pub const FOG = 44.0;
pub const MAXPIL = 3;
pub const MAXGEMA = 3;

// ---------- janela ----------
let win = 0;
let gpu = "wgpu";
let scrw = 900.0;
let scrh = 560.0;
let flen = 320.0;
let meiow = 450.0;
let meiah = 280.0;
let limx = 930.0;
let limy = 590.0;

// ---------- câmera ----------
let camx = 0.0;
let camy = 1.7;
let camz = -24.0;
let yaw = 0.0;
let pitch = 0.0;
let vvel = 0.0;
let no_chao = true;

// base da câmera, pré-computada por frame: frente / direita / cima
let bfx = 0.0;
let bfy = 0.0;
let bfz = 1.0;
let brx = 1.0;
let bry = 0.0;
let brz = 0.0;
let bux = 0.0;
let buy = 1.0;
let buz = 0.0;

// ---------- pilares: centro, meia-base, topo, cor ----------
let pix = [8.0, -9.0, 14.0, -15.0, 2.0, -4.0, 19.0, -20.0];
let piz = [6.0, 10.0, -12.0, -8.0, -18.0, 17.0, 15.0, 2.0];
let piw = [1.2, 1.6, 1.0, 1.4, 2.0, 1.2, 1.0, 1.8];
let pid = [1.2, 1.0, 2.2, 1.4, 1.0, 2.4, 1.0, 1.2];
let pit = [4.0, 2.6, 5.5, 3.2, 2.0, 6.0, 3.6, 4.6];
let pir = [0.35, 1.0, 0.85, 1.0, 0.4, 0.7, 1.0, 0.5];
let pig = [0.9, 0.7, 0.45, 1.0, 0.95, 0.5, 0.8, 0.4];
let pib = [1.0, 0.35, 1.0, 0.5, 0.55, 1.0, 0.4, 0.95];
let NPIL = 8;

// ---------- cristais ----------
let crx = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
let crz = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
let pegas = [0, 0, 0, 0, 0, 0];
let NCRIS = 6;

// ---------- estado da partida ----------
let semente = 20261004;
let pontos = 0;
let onda = 1;
let restantes = 6;
let tempo = 0.0;
let quadros = 0;
let flash = 0.0;
let fps_txt = "fps --";
let ultimo_ms = 0;
let ms_total = 0;
let mx_prev = 0.0;
let my_prev = 0.0;
let mouse_segurando = false;
let fim = false;
let fim_quadro = 0;

// PRNG determinístico: a mesma arena em toda execução.
fn rnd() -> float {
    semente = (semente * 1103515245 + 12345) % 2147483648;
    return (semente as float) / 2147483648.0;
}

// ============ 3D: mundo -> camera -> tela ============
//
// Orçamento de frame: o interpretador custa ~5 us por instrução, então cada
// aresta (rotação + perspectiva + recorte) sai por ~200-240 us. Para ficar em
// 60 fps o frame inteiro tem de caber em ~60 arestas — daí a LOD em desenhar()
// (só o que está perto é desenhado) e a grade enxuta em chao().

// Névoa de um ponto do mundo em profundidade de câmera.
fn nevoa_de(x: float, z: float) -> float {
    let nd = (x - camx) * bfx + (z - camz) * bfz;
    return 1.0 - nd / FOG;
}

// Cor e espessura do traço corrente: ficam em globais para que `linha`
// receba só as duas pontas — cada argumento extra custa ~2 us no frame.
let cr_ = 1.0;
let cg_ = 1.0;
let cb_ = 1.0;
let esp_ = 2.0;

// Aresta entre dois pontos do mundo: gira para o espaço da câmera, recorta no
// plano near, divide pela perspectiva e recorta na moldura (Liang-Barsky).
fn linha(ax: float, ay: float, az: float, bx: float, by: float, bz: float) -> void {
    let dx = ax - camx;
    let dy = ay - camy;
    let dz = az - camz;
    let px = dx * brx + dy * bry + dz * brz;
    let py = dx * bux + dy * buy + dz * buz;
    let pd = dx * bfx + dy * bfy + dz * bfz;
    dx = bx - camx;
    dy = by - camy;
    dz = bz - camz;
    let qx = dx * brx + dy * bry + dz * brz;
    let qy = dx * bux + dy * buy + dz * buz;
    let qd = dx * bfx + dy * bfy + dz * bfz;
    if pd < NEAR {
        if qd < NEAR {
            return;
        }
        let t = (NEAR - pd) / (qd - pd);
        px = px + (qx - px) * t;
        py = py + (qy - py) * t;
        pd = NEAR;
    } else if qd < NEAR {
        let t2 = (NEAR - qd) / (pd - qd);
        qx = qx + (px - qx) * t2;
        qy = qy + (py - qy) * t2;
        qd = NEAR;
    }
    let ka = flen / pd;
    let x1 = meiow + px * ka;
    let y1 = meiah - py * ka;
    let kb = flen / qd;
    let x2 = meiow + qx * kb;
    let y2 = meiah - qy * kb;
    if x1 < -30.0 || x1 > limx || y1 < -30.0 || y1 > limy || x2 < -30.0 || x2 > limx || y2 < -30.0 || y2 > limy {
        // Raspa no plano near -> projeta para dezenas de milhares de pixels e
        // o painter não desenha nada. Recorta na moldura antes de enviar.
        let sx = x2 - x1;
        let sy = y2 - y1;
        let ta = 0.0;
        let tb = 1.0;
        if sx > 0.0 {
            let u0 = (-30.0 - x1) / sx;
            let u1 = (limx - x1) / sx;
            if u0 > ta {
                ta = u0;
            }
            if u1 < tb {
                tb = u1;
            }
        } else if sx < 0.0 {
            let u0 = (limx - x1) / sx;
            let u1 = (-30.0 - x1) / sx;
            if u0 > ta {
                ta = u0;
            }
            if u1 < tb {
                tb = u1;
            }
        } else {
            return;
        }
        if sy > 0.0 {
            let v0 = (-30.0 - y1) / sy;
            let v1 = (limy - y1) / sy;
            if v0 > ta {
                ta = v0;
            }
            if v1 < tb {
                tb = v1;
            }
        } else if sy < 0.0 {
            let v0 = (limy - y1) / sy;
            let v1 = (-30.0 - y1) / sy;
            if v0 > ta {
                ta = v0;
            }
            if v1 < tb {
                tb = v1;
            }
        } else {
            return;
        }
        if ta >= tb {
            return;
        }
        x1 = x1 + sx * ta;
        y1 = y1 + sy * ta;
        x2 = x2 - sx * (1.0 - tb);
        y2 = y2 - sy * (1.0 - tb);
    }
    Canvas::line(win, x1, y1, x2, y2, esp_, cr_, cg_, cb_, 1.0);
}

// Pilarete: aro do topo + 4 montantes (o aro de baixo é o próprio chão).
fn caixa(x: float, z: float, hw: float, ht: float, hd: float, r: float, g: float, b: float, nv: float) -> void {
    cr_ = r * nv;
    cg_ = g * nv;
    cb_ = b * nv;
    let a = x - hw;
    let c = x + hw;
    let f = z - hd;
    let d = z + hd;
    esp_ = 2.6;
    linha(a, ht, f, c, ht, f);
    linha(c, ht, f, c, ht, d);
    linha(c, ht, d, a, ht, d);
    linha(a, ht, d, a, ht, f);
    esp_ = 2.0;
    linha(a, 0.0, f, a, ht, f);
    linha(c, 0.0, f, c, ht, f);
    linha(c, 0.0, d, c, ht, d);
    linha(a, 0.0, d, a, ht, d);
}

// Cristal: octaedro que gira em torno do próprio eixo (aro + 4 faces ao topo).
fn gema(x: float, y: float, z: float, s: float, ang: float, nv: float) -> void {
    cr_ = 0.55 * nv;
    cg_ = 1.0 * nv;
    cb_ = 0.85 * nv;
    let ca = Math::cos(ang);
    let sa = Math::sin(ang);
    let h = s * 1.7;
    let p0x = x + s * ca;
    let p0z = z + s * sa;
    let p1x = x - s * sa;
    let p1z = z + s * ca;
    let p2x = x - s * ca;
    let p2z = z - s * sa;
    let p3x = x + s * sa;
    let p3z = z - s * ca;
    esp_ = 2.4;
    linha(p0x, y, p0z, p1x, y, p1z);
    linha(p1x, y, p1z, p2x, y, p2z);
    linha(p2x, y, p2z, p3x, y, p3z);
    linha(p3x, y, p3z, p0x, y, p0z);
    esp_ = 2.0;
    linha(p0x, y, p0z, x, y + h, z);
    linha(p1x, y, p1z, x, y + h, z);
    linha(p2x, y, p2z, x, y + h, z);
    linha(p3x, y, p3z, x, y + h, z);
}

// ============ Construção do mundo ============

// Espalha os cristais da onda, longe do jogador e fora dos pilares.
fn nova_onda() -> void {
    let i = 0;
    while i < NCRIS {
        let achou = 0;
        let tentativa = 0;
        while achou == 0 && tentativa < 40 {
            let nx = rnd() * 44.0 - 22.0;
            let nz = rnd() * 44.0 - 22.0;
            let dx = nx - camx;
            let dz = nz - camz;
            let longe = Math::sqrt(dx * dx + dz * dz);
            let livre = 1;
            let p = 0;
            while p < NPIL && livre == 1 {
                if nx > pix[p] - piw[p] - 1.5 && nx < pix[p] + piw[p] + 1.5 {
                    if nz > piz[p] - pid[p] - 1.5 && nz < piz[p] + pid[p] + 1.5 {
                        livre = 0;
                    }
                }
                p = p + 1;
            }
            if longe > 7.0 && livre == 1 {
                crx[i] = nx;
                crz[i] = nz;
                pegas[i] = 0;
                achou = 1;
            }
            tentativa = tentativa + 1;
        }
        if achou == 0 {
            crx[i] = -20.0 + i * 8.0;
            crz[i] = 20.0;
            pegas[i] = 0;
        }
        i = i + 1;
    }
    restantes = NCRIS;
}

// Grade do chão: mundo-alinhada, centrada no jogador, poucas linhas.
fn chao() -> void {
    cr_ = 0.2;
    cg_ = 0.4;
    cb_ = 0.6;
    esp_ = 1.0;
    let bx = Math::floor(camx / PASSO) * PASSO;
    let bz = Math::floor(camz / PASSO) * PASSO;
    let k = -NEOX;
    while k <= NEOX {
        let wx = bx + k * PASSO;
        linha(wx, 0.0, camz - EXT, wx, 0.0, camz + EXT);
        k = k + 1;
    }
    let m = -NEOX;
    while m <= NEOX {
        let wz = bz + m * PASSO;
        linha(camx - EXT, 0.0, wz, camx + EXT, 0.0, wz);
        m = m + 1;
    }
}

// Aro superior da arena: dá escala ao espaço sem custar 12 arestas.
fn arena() -> void {
    cr_ = 0.24;
    cg_ = 0.52;
    cb_ = 0.74;
    esp_ = 1.6;
    let q = MEIO - 0.2;
    linha(-q, 5.0, -q, q, 5.0, -q);
    linha(q, 5.0, -q, q, 5.0, q);
    linha(q, 5.0, q, -q, 5.0, q);
    linha(-q, 5.0, q, -q, 5.0, -q);
}

// O jogador estaria batendo? (parede da arena + pilares)
fn bloqueia(nx: float, ny: float, nz: float) -> bool {
    if nx < -MEIO + 0.7 || nx > MEIO - 0.7 {
        return true;
    }
    if nz < -MEIO + 0.7 || nz > MEIO - 0.7 {
        return true;
    }
    let pes = ny - OLHO;
    let i = 0;
    while i < NPIL {
        if pes < pit[i] - 0.25 {
            if nx > pix[i] - piw[i] - 0.5 && nx < pix[i] + piw[i] + 0.5 {
                if nz > piz[i] - pid[i] - 0.5 && nz < piz[i] + pid[i] + 0.5 {
                    return true;
                }
            }
        }
        i = i + 1;
    }
    return false;
}

// ============ Simulação ============

fn olhar(dt: float) -> void {
    if Input::keyDown(win, "left") {
        yaw = yaw - 1.8 * dt;
    }
    if Input::keyDown(win, "right") {
        yaw = yaw + 1.8 * dt;
    }
    if Input::keyDown(win, "up") {
        pitch = pitch - 1.1 * dt;
    }
    if Input::keyDown(win, "down") {
        pitch = pitch + 1.1 * dt;
    }
    let mx = Input::mouseX(win);
    let my = Input::mouseY(win);
    if mouse_segurando {
        yaw = yaw + (mx - mx_prev) * 0.004;
        pitch = pitch + (my - my_prev) * 0.004;
    }
    mx_prev = mx;
    my_prev = my;
    mouse_segurando = Input::mouseDown(win);
    if pitch > 1.2 {
        pitch = 1.2;
    }
    if pitch < -1.2 {
        pitch = -1.2;
    }
}

fn mover(dt: float) -> void {
    let av = 0.0;
    let lad = 0.0;
    if Input::keyDown(win, "w") {
        av = av + 1.0;
    }
    if Input::keyDown(win, "s") {
        av = av - 1.0;
    }
    if Input::keyDown(win, "d") {
        lad = lad + 1.0;
    }
    if Input::keyDown(win, "a") {
        lad = lad - 1.0;
    }
    let vel = 7.0;
    if Input::keyDown(win, "shiftleft") || Input::keyDown(win, "shiftright") {
        vel = 12.0;
    }
    let sy = Math::sin(yaw);
    let cy = Math::cos(yaw);
    let qx = sy * av + cy * lad;
    let qz = cy * av - sy * lad;
    let mag = Math::sqrt(qx * qx + qz * qz);
    if mag > 0.01 {
        let nx = camx + qx / mag * vel * dt;
        if !bloqueia(nx, camy, camz) {
            camx = nx;
        }
        let nz = camz + qz / mag * vel * dt;
        if !bloqueia(camx, camy, nz) {
            camz = nz;
        }
    }
    if Input::keyDown(win, "space") && no_chao {
        vvel = 6.4;
        no_chao = false;
    }
    vvel = vvel - 17.0 * dt;
    let ny = camy + vvel * dt;
    if ny <= OLHO {
        camy = OLHO;
        vvel = 0.0;
        no_chao = true;
    } else {
        camy = ny;
    }
}

fn coletar() -> void {
    let i = 0;
    while i < NCRIS {
        if pegas[i] == 0 {
            let dx = crx[i] - camx;
            let dz = crz[i] - camz;
            if dx * dx + dz * dz < 2.4 {
                pegas[i] = 1;
                restantes = restantes - 1;
                pontos = pontos + 100;
                flash = 1.0;
                tempo = tempo - 3.5;
                if tempo < 0.0 {
                    tempo = 0.0;
                }
            }
        }
        i = i + 1;
    }
    if restantes == 0 {
        onda = onda + 1;
        nova_onda();
    }
}

// Base ortonormal da câmera + distância focal (FOV vertical de 60 graus).
fn calcular_base() -> void {
    let cp = Math::cos(pitch);
    let sp = Math::sin(pitch);
    let cy = Math::cos(yaw);
    let sy = Math::sin(yaw);
    bfx = sy * cp;
    bfy = -sp;
    bfz = cy * cp;
    brx = cy;
    bry = 0.0;
    brz = -sy;
    bux = sp * sy;
    buy = cp;
    buz = sp * cy;
    flen = scrh * 0.5 / Math::tan(0.524);
    meiow = scrw * 0.5;
    meiah = scrh * 0.5;
    limx = scrw + 30.0;
    limy = scrh + 30.0;
}

// ============ Desenho ============

fn desenhar() -> void {
    Canvas::clear(win, 0.03, 0.04, 0.08, 1.0);
    chao();
    arena();
    let i = 0;
    let feitos = 0;
    while i < NPIL && feitos < MAXPIL {
        let dx = pix[i] - camx;
        let dz = piz[i] - camz;
        if dx * dx + dz * dz < 400.0 {
            let nv = nevoa_de(pix[i], piz[i]);
            if nv > 0.1 {
                caixa(pix[i], piz[i], piw[i], pit[i], pid[i], pir[i], pig[i], pib[i], nv);
                feitos = feitos + 1;
            }
        }
        i = i + 1;
    }
    let j = 0;
    let gemas = 0;
    while j < NCRIS && gemas < MAXGEMA {
        if pegas[j] == 0 {
            let dx = crx[j] - camx;
            let dz = crz[j] - camz;
            if dx * dx + dz * dz < 484.0 {
                let nv = nevoa_de(crx[j], crz[j]);
                if nv > 0.15 {
                    let flutu = Math::sin(tempo * 2.2 + j) * 0.22;
                    gema(crx[j], 1.35 + flutu, crz[j], 0.55, tempo * 1.6 + crx[j], nv);
                    gemas = gemas + 1;
                }
            }
        }
        j = j + 1;
    }
}

fn hud() -> void {
    let hr = 1.0;
    let hg = 1.0;
    let hb = 1.0;
    if flash > 0.01 {
        hr = 1.0;
        hg = 0.55 + 0.45 * flash;
        hb = 0.3;
    }
    let cxm = scrw * 0.5;
    let cym = scrh * 0.5;
    Canvas::line(win, cxm - 9.0, cym, cxm + 9.0, cym, 1.5, 0.85, 0.95, 1.0, 0.8);
    Canvas::text(win, 16.0, 12.0, "COLETOR 3D - LEX", 21, hr, hg, hb, 1.0);
    Canvas::text(win, 16.0, 40.0, "CRISTAIS " + restantes + " / " + NCRIS, 18, 0.6, 1.0, 0.9, 1.0);
    Canvas::text(win, 16.0, 64.0, "PONTOS " + pontos, 18, 1.0, 0.85, 0.4, 1.0);
    Canvas::text(win, 16.0, 88.0, "ONDA " + onda + "   " + fps_txt, 15, 0.7, 0.75, 0.9, 1.0);
    let sobra = LIMITE - tempo;
    if sobra < 0.0 {
        sobra = 0.0;
    }
    Canvas::text(win, scrw - 170.0, 12.0, "TEMPO " + (sobra as int), 21, 1.0, 0.45, 0.45, 1.0);
    Canvas::text(win, 16.0, scrh - 46.0, "WASD mover  SHIFT correr  ESPACO pular  setas ou mouse virar  ESC sair", 13, 0.6, 0.65, 0.8, 1.0);
    Canvas::text(win, 16.0, scrh - 66.0, "GPU: " + gpu, 13, 0.5, 0.85, 0.62, 1.0);
}

fn tela_fim() -> void {
    Canvas::clear(win, 0.02, 0.03, 0.06, 1.0);
    Canvas::text(win, scrw * 0.5 - 120.0, scrh * 0.5 - 44.0, "FIM DE JOGO", 34, 1.0, 0.5, 0.5, 1.0);
    Canvas::text(win, scrw * 0.5 - 130.0, scrh * 0.5 + 8.0, "PONTOS " + pontos + "   ONDA " + onda, 22, 0.9, 0.9, 1.0, 1.0);
}

// ============ Loop principal ============

pub fn main() -> void {
    win = Window::create(Title { title: "Coletor 3D - Lex", width: 900, height: 560 });
    scrw = Window::width(win);
    scrh = Window::height(win);
    ultimo_ms = Time::now_unix_ms();
    mx_prev = scrw * 0.5;
    my_prev = scrh * 0.5;
    nova_onda();

    while !Window::shouldClose(win) {
        let agora = Time::now_unix_ms();
        let ms = agora - ultimo_ms;
        ultimo_ms = agora;
        let dt = ms / 1000.0;
        if dt > 0.05 {
            dt = 0.05;
        }
        if dt < 0.001 {
            dt = 0.001;
        }
        quadros = quadros + 1;
        ms_total = ms_total + ms;
        if quadros % 20 == 0 && ms > 0 {
            fps_txt = "fps " + (1000 / ms);
            scrw = Window::width(win);
            scrh = Window::height(win);
        }
        if gpu == "wgpu" && quadros > 3 {
            gpu = Window::backend(win);
        }

        if Input::keyDown(win, "escape") {
            break;
        }

        if !fim {
            tempo = tempo + dt;
            if flash > 0.0 {
                flash = flash - dt * 2.0;
            }
            olhar(dt);
            mover(dt);
            coletar();
            if tempo > LIMITE || quadros > FRAMES_MAX {
                fim = true;
                fim_quadro = quadros;
            }
        }

        calcular_base();
        if fim {
            tela_fim();
            if quadros > fim_quadro + 150 {
                break;
            }
        } else {
            desenhar();
            hud();
        }

        Window::present(win);
    }

    Window::close(win);
    let media = 0;
    if quadros > 0 {
        media = (quadros * 1000) / ms_total;
    }
    Console::log("Coletor 3D — " + pontos.to_string() + " pontos, onda " + onda.to_string() + ", " + quadros.to_string() + " frames, media " + media.to_string() + " fps, GPU: " + gpu);
}
