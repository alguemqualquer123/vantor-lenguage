module tests.test_990;

@Getter
class User_990 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_990() {
    let u = User_990(990, "User_990")
    assert(u.getId() == 990)
}


pub fn main() {
    test_990()
}
