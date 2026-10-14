use crate::model::{Category, CheckResult, PasswdEntry, Status}; //imports de los tipos

mod model;

fn main() {
    //println!("Cenda: Security Auditor");

    let check_test = CheckResult{
        id: String::from("first_user_check"),
        category: Category::Users,
        status: Status::Pass,
        message: String::from("First User Check"),
        recommendation: None
    };

    let user_test = PasswdEntry{
        username: String::from("root"),
        uid: 0,
        gid: 0,
        home: String::from("/root"),
        shell: String::from("/bin/bash")
    };

    println!("Check ID: {}", check_test.id);
    println!("User: {}", user_test.username);
    println!("UID: {}", user_test.uid);
    
}
