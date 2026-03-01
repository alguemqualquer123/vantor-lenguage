module tests.test_798;

@Getter
class User_798 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_798() {
    let u = User_798(798, "User_798")
    assert(u.getId() == 798)
}


pub fn main() {
    test_798()
}
