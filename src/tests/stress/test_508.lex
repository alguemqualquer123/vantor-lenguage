module tests.test_508;

@Getter
class User_508 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_508() {
    let u = User_508(508, "User_508")
    assert(u.getId() == 508)
}


pub fn main() {
    test_508()
}
