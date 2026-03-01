module tests.test_190;

@Getter
class User_190 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_190() {
    let u = User_190(190, "User_190")
    assert(u.getId() == 190)
}


pub fn main() {
    test_190()
}
