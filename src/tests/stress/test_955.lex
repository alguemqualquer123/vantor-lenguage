module tests.test_955;

@Getter
class User_955 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_955() {
    let u = User_955(955, "User_955")
    assert(u.getId() == 955)
}


pub fn main() {
    test_955()
}
