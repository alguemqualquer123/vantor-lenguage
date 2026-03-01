module tests.test_914;

@Getter
class User_914 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_914() {
    let u = User_914(914, "User_914")
    assert(u.getId() == 914)
}


pub fn main() {
    test_914()
}
