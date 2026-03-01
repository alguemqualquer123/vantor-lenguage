module tests.test_496;

@Getter
class User_496 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_496() {
    let u = User_496(496, "User_496")
    assert(u.getId() == 496)
}


pub fn main() {
    test_496()
}
