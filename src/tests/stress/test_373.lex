module tests.test_373;

@Getter
class User_373 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_373() {
    let u = User_373(373, "User_373")
    assert(u.getId() == 373)
}


pub fn main() {
    test_373()
}
