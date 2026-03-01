module tests.test_323;

@Getter
class User_323 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_323() {
    let u = User_323(323, "User_323")
    assert(u.getId() == 323)
}


pub fn main() {
    test_323()
}
