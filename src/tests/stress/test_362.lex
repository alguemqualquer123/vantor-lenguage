module tests.test_362;

@Getter
class User_362 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_362() {
    let u = User_362(362, "User_362")
    assert(u.getId() == 362)
}


pub fn main() {
    test_362()
}
