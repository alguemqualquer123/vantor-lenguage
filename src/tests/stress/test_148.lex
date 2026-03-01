module tests.test_148;

@Getter
class User_148 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_148() {
    let u = User_148(148, "User_148")
    assert(u.getId() == 148)
}


pub fn main() {
    test_148()
}
