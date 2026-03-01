module tests.test_510;

@Getter
class User_510 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_510() {
    let u = User_510(510, "User_510")
    assert(u.getId() == 510)
}


pub fn main() {
    test_510()
}
