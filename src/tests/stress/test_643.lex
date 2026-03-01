module tests.test_643;

@Getter
class User_643 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_643() {
    let u = User_643(643, "User_643")
    assert(u.getId() == 643)
}


pub fn main() {
    test_643()
}
