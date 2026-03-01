module tests.test_822;

@Getter
class User_822 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_822() {
    let u = User_822(822, "User_822")
    assert(u.getId() == 822)
}


pub fn main() {
    test_822()
}
