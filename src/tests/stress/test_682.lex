module tests.test_682;

@Getter
class User_682 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_682() {
    let u = User_682(682, "User_682")
    assert(u.getId() == 682)
}


pub fn main() {
    test_682()
}
