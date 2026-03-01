module tests.test_295;

@Getter
class User_295 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_295() {
    let u = User_295(295, "User_295")
    assert(u.getId() == 295)
}


pub fn main() {
    test_295()
}
