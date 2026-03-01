import core.net.Http;
import core.io.Console;

struct User {
    id: i32;
    name: String;
}

@Get("/")
pub async fn root() -> String {
    return "Welcome to the decorated API!";
}

@Get("/users")
pub async fn list_users() -> List<User> {
    let users = List<User>::new();
    users.add(User { id: 1, name: "John" });
    return users;
}

pub async fn get_user(id: i32) -> User {
    return User { id, name: "John".to_string() };
}

@Post("/users")
pub async fn create_user(payload: User) -> User {
    return payload;
}

pub async fn main() -> void {
    // Example of pipe operator
    let message = "Hello" |> String::toUpper() |> String::append(" WORLD!");
    Console.writeLine(message);

    // The compiler/runner will automatically register decorated functions
    await Http::serve("0.0.0.0:3000");
}
