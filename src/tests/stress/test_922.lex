module tests.test_922;

@Getter
class User_922 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_922() {
    let u = User_922(922, "User_922")
    assert(u.getId() == 922)
}


pub fn main() {
    test_922()
}
