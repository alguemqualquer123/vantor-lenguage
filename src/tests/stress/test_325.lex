module tests.test_325;

@Getter
class User_325 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_325() {
    let u = User_325(325, "User_325")
    assert(u.getId() == 325)
}


pub fn main() {
    test_325()
}
