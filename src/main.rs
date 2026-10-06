use crate::model::{Category, CheckResult, Status}; //imports de los tipos

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

    println!("Check ID: {}", check_test.id);
}
