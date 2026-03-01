module tests.test_458;

@Getter
class User_458 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_458() {
    let u = User_458(458, "User_458")
    assert(u.getId() == 458)
}


pub fn main() {
    test_458()
}
