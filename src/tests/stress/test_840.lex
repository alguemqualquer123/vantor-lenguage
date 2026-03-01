module tests.test_840;

@Getter
class User_840 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_840() {
    let u = User_840(840, "User_840")
    assert(u.getId() == 840)
}


pub fn main() {
    test_840()
}
