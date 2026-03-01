module tests.test_864;

@Getter
class User_864 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_864() {
    let u = User_864(864, "User_864")
    assert(u.getId() == 864)
}


pub fn main() {
    test_864()
}
