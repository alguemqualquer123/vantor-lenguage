module tests.test_266;

@Getter
class User_266 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_266() {
    let u = User_266(266, "User_266")
    assert(u.getId() == 266)
}


pub fn main() {
    test_266()
}
