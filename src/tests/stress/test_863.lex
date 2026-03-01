module tests.test_863;

@Getter
class User_863 {
    pub id: i32;
    pub name: String;
    
    pub constructor(id: i32, name: String) {
        self.id = id;
        self.name = name;
    }
}

fn test_863() {
    let u = User_863(863, "User_863")
    assert(u.getId() == 863)
}


pub fn main() {
    test_863()
}
