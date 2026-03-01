module tests.test_279;

@Getter
class User_279 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_279() {
    let u = User_279(279, "User_279")
    assert(u.getId() == 279)
}


pub fn main() {
    test_279()
}
