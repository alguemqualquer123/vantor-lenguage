module tests.test_906;

@Getter
class User_906 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_906() {
    let u = User_906(906, "User_906")
    assert(u.getId() == 906)
}


pub fn main() {
    test_906()
}
