module tests.test_885;

@Getter
class User_885 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_885() {
    let u = User_885(885, "User_885")
    assert(u.getId() == 885)
}


pub fn main() {
    test_885()
}
