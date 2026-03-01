module tests.test_471;

@Getter
class User_471 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_471() {
    let u = User_471(471, "User_471")
    assert(u.getId() == 471)
}


pub fn main() {
    test_471()
}
