module tests.test_722;

@Getter
class User_722 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_722() {
    let u = User_722(722, "User_722")
    assert(u.getId() == 722)
}


pub fn main() {
    test_722()
}
