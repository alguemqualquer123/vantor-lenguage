module tests.test_395;

@Getter
class User_395 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_395() {
    let u = User_395(395, "User_395")
    assert(u.getId() == 395)
}


pub fn main() {
    test_395()
}
