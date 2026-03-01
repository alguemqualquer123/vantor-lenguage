module tests.test_4;

@Getter
class User_4 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_4() {
    let u = User_4(4, "User_4")
    assert(u.getId() == 4)
}


pub fn main() {
    test_4()
}
