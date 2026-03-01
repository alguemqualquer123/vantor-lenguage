module tests.test_73;

@Getter
class User_73 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_73() {
    let u = User_73(73, "User_73")
    assert(u.getId() == 73)
}


pub fn main() {
    test_73()
}
