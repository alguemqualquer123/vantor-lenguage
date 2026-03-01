module tests.test_291;

@Getter
class User_291 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_291() {
    let u = User_291(291, "User_291")
    assert(u.getId() == 291)
}


pub fn main() {
    test_291()
}
