module tests.test_402;

@Getter
class User_402 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_402() {
    let u = User_402(402, "User_402")
    assert(u.getId() == 402)
}


pub fn main() {
    test_402()
}
