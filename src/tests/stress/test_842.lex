module tests.test_842;

@Getter
class User_842 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_842() {
    let u = User_842(842, "User_842")
    assert(u.getId() == 842)
}


pub fn main() {
    test_842()
}
