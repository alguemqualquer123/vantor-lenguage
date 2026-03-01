module tests.test_683;

@Getter
class User_683 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_683() {
    let u = User_683(683, "User_683")
    assert(u.getId() == 683)
}


pub fn main() {
    test_683()
}
