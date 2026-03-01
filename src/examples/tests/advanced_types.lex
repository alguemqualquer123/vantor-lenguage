import core.io.Console;

// Aliases de tipos
type UserId = u64;
type Username = String;

// Interface definindo um contrato
interface Identifiable {
    fn get_id() -> UserId;
    fn get_name() -> String;
}

// Classe implementando a interface
class User implements Identifiable {
    pub id: UserId;
    pub name: Username;
    pub email: String;

    pub constructor(id: UserId, name: Username, email: String) {
        self.id = id;
        self.name = name;
        self.email = email;
    }

    pub fn get_id() -> UserId {
        return self.id;
    }

    pub fn get_name() -> String {
        return self.name;
    }

    pub fn to_string() -> String {
        return "User(id=" + self.id as String + ", name=" + self.name + ")";
    }
}

// Struct genérica
struct Box<T> {
    value: T;
}

pub fn main() -> void {
    let user = User {
        id: 100u64,
        name: "Developer",
        email: "dev@lexicon.org",
    };

    Console.writeLine("User Info: " + user.to_string());
    
    // Testando interface
    let id: Identifiable = user;
    Console.writeLine("Identifiable Name: " + id.get_name());
    
    // Testando Generics
    let intBox = Box<i32> { value: 42 };
    Console.writeLine("Box Value: " + intBox.value as String);
}
