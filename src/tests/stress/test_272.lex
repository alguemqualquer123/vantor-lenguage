module tests.test_272;

@Getter
class User_272 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_272() {
    let u = User_272(272, "User_272")
    assert(u.getId() == 272)
}


pub fn main() {
    test_272()
}
