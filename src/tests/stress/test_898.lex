module tests.test_898;

@Getter
class User_898 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_898() {
    let u = User_898(898, "User_898")
    assert(u.getId() == 898)
}


pub fn main() {
    test_898()
}
