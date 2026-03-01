module tests.test_202;

@Getter
class User_202 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_202() {
    let u = User_202(202, "User_202")
    assert(u.getId() == 202)
}


pub fn main() {
    test_202()
}
