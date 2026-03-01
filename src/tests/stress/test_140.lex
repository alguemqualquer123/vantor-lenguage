module tests.test_140;

@Getter
class User_140 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_140() {
    let u = User_140(140, "User_140")
    assert(u.getId() == 140)
}


pub fn main() {
    test_140()
}
