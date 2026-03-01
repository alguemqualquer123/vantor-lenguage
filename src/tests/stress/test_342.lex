module tests.test_342;

@Getter
class User_342 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_342() {
    let u = User_342(342, "User_342")
    assert(u.getId() == 342)
}


pub fn main() {
    test_342()
}
