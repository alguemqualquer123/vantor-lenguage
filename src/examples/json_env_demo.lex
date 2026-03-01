import core.io.Console;
import core.json.Json;
import core.env.Env;



struct Config {
    port: i32;
    apiKey: String;
    debug: bool;
}

pub fn main() -> void {
    // 1. Carregando variáveis de ambiente (.env)
    let port = Env::get("PORT") |> String::toInt();
    let apiKey = Env::get("API_KEY");
    
    Console::writeLine("🚀 Configuração carregada do .env:");
    Console::writeLine("Porta: " + port as String);

    // 2. Usando JSON
    let jsonStr = "{ \"status\": \"success\", \"code\": 200 }";
    let data = Json::parse(jsonStr);
    
    Console::writeLine("\n📦 Resposta JSON:");
    Console::writeLine(data.toString());

    // 3. Criando Struct com dados mistos
    Console::writeLine("\n🔧 Criando objeto Config...");
    let config = Config {
        port,
        apiKey,
        debug: true
    };
    
    Console::writeLine("\n✅ Objeto Config criado com sucesso!", inspect(config););
}
