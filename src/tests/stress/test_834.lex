module tests.test_834;

@Getter
class User_834 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_834() {
    let u = User_834(834, "User_834")
    assert(u.getId() == 834)
}


pub fn main() {
    test_834()
}
