type UserId = u64;

interface Entity {
    id: UserId;
    name: String;
}

class User implements Entity {
    pub id: UserId;
    pub name: String;
    pub email: String;

    pub constructor(id: UserId, name: String, email: String) {
        self.id = id;
        self.name = name;
        self.email = email;
    }

    pub fn to_string() -> String {
        return "User(id: " + self.id as String + ", name: " + self.name + ")";
    }
}

pub fn main() -> void {
    let user = User {
        id: 100u64,
        name: "Developer",
        email: "dev@lexicon.org",
    };

    Console.writeLine(user.to_string());
}
