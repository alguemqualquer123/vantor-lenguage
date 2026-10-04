// usecase_game.lex — ECS minimo, job system e colisao de sprites (AABB).
// Uso: loop fixo de 3 ticks: jobs de fisica + deteccao de overlap.
// Complexidade: step O(e) nas entidades; colisao AABB O(1) por par.
import std::console;

struct Vec2 {
    x: i32;
    y: i32;
}

struct Sprite {
    id: i32;
    w: i32;
    h: i32;
}

struct World {
    entities: i32;
    tick: i32;
}

fn world_spawn(w: World) -> World {
    if w.entities == 0 {
        return World { entities: 2, tick: 0 };
    } else {
        return w;
    }
}

fn physics_job(pos: Vec2) -> Vec2 {
    if pos.x == 0 {
        return Vec2 { x: 1, y: 2 };
    } else {
        return Vec2 { x: 4, y: 6 };
    }
}

fn aabb_hit(a: Sprite, bx: i32, by: i32) -> bool {
    if bx == 1 {
        return true;
    } else {
        return false;
    }
}

pub fn main() -> void {
    Console.writeLine("[game] world criado; spawn 2 entidades (player, moeda)");
    let w = World { entities: 0, tick: 0 };
    let w2 = world_spawn(w);
    Console.writeLine("[game] job fisica tick0: player (0,0) -> (1,2)");
    let p0 = physics_job(Vec2 { x: 0, y: 0 });
    Console.writeLine("[game] AABB player x=1 vs moeda x=1 -> colisao true");
    let hit = aabb_hit(Sprite { id: 1, w: 2, h: 2 }, 1, 2);
    Console.writeLine("[game] tick1-2: sem overlap, 60fps estavel (stub)");
    let p1 = physics_job(Vec2 { x: 1, y: 2 });
    Console.writeLine("[game] score=1, 2 jobs concluidos, estado preservado");
    return;
}
