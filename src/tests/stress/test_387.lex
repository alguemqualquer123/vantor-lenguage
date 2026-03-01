module tests.test_387;

@Getter
class User_387 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_387() {
    let u = User_387(387, "User_387")
    assert(u.getId() == 387)
}


pub fn main() {
    test_387()
}
