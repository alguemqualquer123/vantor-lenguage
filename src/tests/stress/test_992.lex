module tests.test_992;

@Getter
class User_992 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_992() {
    let u = User_992(992, "User_992")
    assert(u.getId() == 992)
}


pub fn main() {
    test_992()
}
