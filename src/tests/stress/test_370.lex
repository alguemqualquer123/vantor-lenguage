module tests.test_370;

@Getter
class User_370 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_370() {
    let u = User_370(370, "User_370")
    assert(u.getId() == 370)
}


pub fn main() {
    test_370()
}
