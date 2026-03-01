module tests.test_340;

@Getter
class User_340 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_340() {
    let u = User_340(340, "User_340")
    assert(u.getId() == 340)
}


pub fn main() {
    test_340()
}
