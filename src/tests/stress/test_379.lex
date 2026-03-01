module tests.test_379;

@Getter
class User_379 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_379() {
    let u = User_379(379, "User_379")
    assert(u.getId() == 379)
}


pub fn main() {
    test_379()
}
