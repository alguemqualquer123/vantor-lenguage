module tests.test_98;

@Getter
class User_98 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_98() {
    let u = User_98(98, "User_98")
    assert(u.getId() == 98)
}


pub fn main() {
    test_98()
}
