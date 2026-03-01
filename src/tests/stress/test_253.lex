module tests.test_253;

@Getter
class User_253 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_253() {
    let u = User_253(253, "User_253")
    assert(u.getId() == 253)
}


pub fn main() {
    test_253()
}
