module tests.test_505;

@Getter
class User_505 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_505() {
    let u = User_505(505, "User_505")
    assert(u.getId() == 505)
}


pub fn main() {
    test_505()
}
