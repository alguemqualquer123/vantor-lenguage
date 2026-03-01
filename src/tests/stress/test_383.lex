module tests.test_383;

@Getter
class User_383 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_383() {
    let u = User_383(383, "User_383")
    assert(u.getId() == 383)
}


pub fn main() {
    test_383()
}
