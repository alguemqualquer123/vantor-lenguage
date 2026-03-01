module tests.test_277;

@Getter
class User_277 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_277() {
    let u = User_277(277, "User_277")
    assert(u.getId() == 277)
}


pub fn main() {
    test_277()
}
