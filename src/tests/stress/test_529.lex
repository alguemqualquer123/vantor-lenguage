module tests.test_529;

@Getter
class User_529 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_529() {
    let u = User_529(529, "User_529")
    assert(u.getId() == 529)
}


pub fn main() {
    test_529()
}
