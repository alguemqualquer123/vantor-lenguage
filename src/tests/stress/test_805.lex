module tests.test_805;

@Getter
class User_805 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_805() {
    let u = User_805(805, "User_805")
    assert(u.getId() == 805)
}


pub fn main() {
    test_805()
}
