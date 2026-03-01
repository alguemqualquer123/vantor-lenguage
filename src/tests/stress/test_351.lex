module tests.test_351;

@Getter
class User_351 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_351() {
    let u = User_351(351, "User_351")
    assert(u.getId() == 351)
}


pub fn main() {
    test_351()
}
