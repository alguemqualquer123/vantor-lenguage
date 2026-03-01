module tests.test_454;

@Getter
class User_454 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_454() {
    let u = User_454(454, "User_454")
    assert(u.getId() == 454)
}


pub fn main() {
    test_454()
}
