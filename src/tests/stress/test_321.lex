module tests.test_321;

@Getter
class User_321 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_321() {
    let u = User_321(321, "User_321")
    assert(u.getId() == 321)
}


pub fn main() {
    test_321()
}
