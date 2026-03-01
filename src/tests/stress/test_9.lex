module tests.test_9;

@Getter
class User_9 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_9() {
    let u = User_9(9, "User_9")
    assert(u.getId() == 9)
}


pub fn main() {
    test_9()
}
