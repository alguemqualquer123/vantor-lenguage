module tests.test_201;

@Getter
class User_201 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_201() {
    let u = User_201(201, "User_201")
    assert(u.getId() == 201)
}


pub fn main() {
    test_201()
}
