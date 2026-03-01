module tests.test_596;

@Getter
class User_596 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_596() {
    let u = User_596(596, "User_596")
    assert(u.getId() == 596)
}


pub fn main() {
    test_596()
}
