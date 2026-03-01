module tests.test_979;

@Getter
class User_979 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_979() {
    let u = User_979(979, "User_979")
    assert(u.getId() == 979)
}


pub fn main() {
    test_979()
}
