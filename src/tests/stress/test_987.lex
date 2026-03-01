module tests.test_987;

@Getter
class User_987 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_987() {
    let u = User_987(987, "User_987")
    assert(u.getId() == 987)
}


pub fn main() {
    test_987()
}
