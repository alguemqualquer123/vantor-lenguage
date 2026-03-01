module tests.test_809;

@Getter
class User_809 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_809() {
    let u = User_809(809, "User_809")
    assert(u.getId() == 809)
}


pub fn main() {
    test_809()
}
