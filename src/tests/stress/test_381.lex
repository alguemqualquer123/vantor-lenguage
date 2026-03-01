module tests.test_381;

@Getter
class User_381 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_381() {
    let u = User_381(381, "User_381")
    assert(u.getId() == 381)
}


pub fn main() {
    test_381()
}
