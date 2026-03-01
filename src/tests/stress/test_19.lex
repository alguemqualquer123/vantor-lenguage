module tests.test_19;

@Getter
class User_19 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_19() {
    let u = User_19(19, "User_19")
    assert(u.getId() == 19)
}


pub fn main() {
    test_19()
}
