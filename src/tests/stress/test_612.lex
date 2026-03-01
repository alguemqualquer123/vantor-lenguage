module tests.test_612;

@Getter
class User_612 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_612() {
    let u = User_612(612, "User_612")
    assert(u.getId() == 612)
}


pub fn main() {
    test_612()
}
