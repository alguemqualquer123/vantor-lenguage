module tests.test_369;

@Getter
class User_369 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_369() {
    let u = User_369(369, "User_369")
    assert(u.getId() == 369)
}


pub fn main() {
    test_369()
}
