module tests.test_841;

@Getter
class User_841 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_841() {
    let u = User_841(841, "User_841")
    assert(u.getId() == 841)
}


pub fn main() {
    test_841()
}
