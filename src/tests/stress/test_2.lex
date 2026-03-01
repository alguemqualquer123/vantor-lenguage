module tests.test_2;

@Getter
class User_2 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_2() {
    let u = User_2(2, "User_2")
    assert(u.getId() == 2)
}


pub fn main() {
    test_2()
}
