module tests.test_422;

@Getter
class User_422 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_422() {
    let u = User_422(422, "User_422")
    assert(u.getId() == 422)
}


pub fn main() {
    test_422()
}
