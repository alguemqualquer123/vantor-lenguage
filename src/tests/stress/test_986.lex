module tests.test_986;

@Getter
class User_986 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_986() {
    let u = User_986(986, "User_986")
    assert(u.getId() == 986)
}


pub fn main() {
    test_986()
}
