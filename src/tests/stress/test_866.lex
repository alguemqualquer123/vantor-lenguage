module tests.test_866;

@Getter
class User_866 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_866() {
    let u = User_866(866, "User_866")
    assert(u.getId() == 866)
}


pub fn main() {
    test_866()
}
