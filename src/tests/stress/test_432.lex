module tests.test_432;

@Getter
class User_432 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_432() {
    let u = User_432(432, "User_432")
    assert(u.getId() == 432)
}


pub fn main() {
    test_432()
}
