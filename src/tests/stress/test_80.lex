module tests.test_80;

@Getter
class User_80 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_80() {
    let u = User_80(80, "User_80")
    assert(u.getId() == 80)
}


pub fn main() {
    test_80()
}
