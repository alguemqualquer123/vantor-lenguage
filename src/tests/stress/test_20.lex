module tests.test_20;

@Getter
class User_20 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_20() {
    let u = User_20(20, "User_20")
    assert(u.getId() == 20)
}


pub fn main() {
    test_20()
}
