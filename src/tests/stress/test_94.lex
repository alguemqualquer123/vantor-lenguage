module tests.test_94;

@Getter
class User_94 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_94() {
    let u = User_94(94, "User_94")
    assert(u.getId() == 94)
}


pub fn main() {
    test_94()
}
