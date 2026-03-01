module tests.test_852;

@Getter
class User_852 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_852() {
    let u = User_852(852, "User_852")
    assert(u.getId() == 852)
}


pub fn main() {
    test_852()
}
