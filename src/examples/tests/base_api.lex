import core.net.Http;
import core.io.Console;

struct User {
    id: i32;
    name: String;
    email: String;
}

struct ApiResponse<T> {
    success: bool;
    data: T;
}

@Get("/")
pub async fn root() -> String {
    return "Welcome to Lexicon Base API";
}

@Get("/users")
pub async fn get_users() -> ApiResponse<List<User>> {
    let users = List<User>::new();
    users.add(User { id: 1, name: "Alice", email: "alice@lexicon.dev" });
    users.add(User { id: 2, name: "Bob", email: "bob@lexicon.dev" });
    
    return ApiResponse<List<User>> {
        success: true,
        data: users
    };
}

@Post("/users")
pub async fn create_user(payload: User) -> ApiResponse<User> {
    // Simulação de criação
    Console.writeLine("Creating user: " + payload.name);
    
    return ApiResponse<User> {
        success: true,
        data: payload
    };
}

pub async fn main() -> void {
    Console.writeLine("🚀 Starting Lexicon Base API Example...");
    
    // O compilador Lexicon detecta automaticamente as funções decoradas com @Get/@Post
    await Http::serve("0.0.0.0:3000");
}
