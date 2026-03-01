module tests.test_451;

@Getter
class User_451 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_451() {
    let u = User_451(451, "User_451")
    assert(u.getId() == 451)
}


pub fn main() {
    test_451()
}
