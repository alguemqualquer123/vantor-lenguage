module tests.test_764;

@Getter
class User_764 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_764() {
    let u = User_764(764, "User_764")
    assert(u.getId() == 764)
}


pub fn main() {
    test_764()
}
