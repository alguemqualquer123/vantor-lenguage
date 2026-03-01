module tests.test_101;

@Getter
class User_101 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_101() {
    let u = User_101(101, "User_101")
    assert(u.getId() == 101)
}


pub fn main() {
    test_101()
}
