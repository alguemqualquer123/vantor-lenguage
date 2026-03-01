import core.io.Console;

// 1. Definindo um Algebraic Data Type (ADT)
// Enums no Lexicon podem carregar dados complexos (Sum Types)
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
    Square(f64),
    Point
}

// 2. Definindo um Sum Type para tratamento de erros funcional
enum Result<T, E> {
    Ok(T),
    Err(E),
    Loading
}

pub fn main() -> void {
    let my_shape = Shape::Circle(15.5);
    
    Console::writeLine("🎨 Shape ADT Demo");
    
    // 3. Pattern Matching Exaustivo
    // O compilador verifica se todos os casos foram tratados
    match my_shape {
        Shape::Circle(radius) => {
            Console::writeLine("Círculo com raio: " + radius as String);
        }
        Shape::Rectangle(w, h) => {
            Console::writeLine("Retângulo: " + w as String + "x" + h as String);
        }
        Shape::Square(s) => {
            Console::writeLine("Quadrado de lado: " + s as String);
        }
        Shape::Point => {
            Console::writeLine("Apenas um ponto no espaço.");
        }
    }

    // 4. Usando Result ADT
    let status = Result::Ok("Dados processados com sucesso!");
    
    match status {
        Result::Ok(msg) => Console::writeLine("✅ Sucesso: " + msg),
        Result::Err(e)  => Console::writeLine("❌ Erro: " + e),
        Result::Loading => Console::writeLine("⏳ Carregando..."),
    }
}
