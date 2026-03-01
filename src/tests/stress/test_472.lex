module tests.test_472;

@Getter
class User_472 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_472() {
    let u = User_472(472, "User_472")
    assert(u.getId() == 472)
}


pub fn main() {
    test_472()
}
