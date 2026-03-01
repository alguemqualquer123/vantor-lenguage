module tests.test_704;

@Getter
class User_704 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_704() {
    let u = User_704(704, "User_704")
    assert(u.getId() == 704)
}


pub fn main() {
    test_704()
}
