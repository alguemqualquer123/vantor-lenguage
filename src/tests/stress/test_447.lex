module tests.test_447;

@Getter
class User_447 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_447() {
    let u = User_447(447, "User_447")
    assert(u.getId() == 447)
}


pub fn main() {
    test_447()
}
