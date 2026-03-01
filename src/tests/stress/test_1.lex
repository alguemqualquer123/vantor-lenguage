module tests.test_1;

@Getter
class User_1 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_1() {
    let u = User_1(1, "User_1")
    assert(u.getId() == 1)
}


pub fn main() {
    test_1()
}
