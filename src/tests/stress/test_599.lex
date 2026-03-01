module tests.test_599;

@Getter
class User_599 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_599() {
    let u = User_599(599, "User_599")
    assert(u.getId() == 599)
}


pub fn main() {
    test_599()
}
