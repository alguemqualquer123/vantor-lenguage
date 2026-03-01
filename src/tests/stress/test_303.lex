module tests.test_303;

@Getter
class User_303 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_303() {
    let u = User_303(303, "User_303")
    assert(u.getId() == 303)
}


pub fn main() {
    test_303()
}
