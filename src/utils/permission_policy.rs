#[derive(PartialEq, Clone)]
pub struct Policy {
    pub user_policy: Vec<String>,
    pub courier_policy: Vec<String>,
    pub admin_policy: Vec<String>,
    pub analyst_policy: Vec<String>,
}

impl Policy {
    pub fn build() -> Self {
        Policy {
            user_policy: vec![
                "USER".to_owned(),
                "COURIER".to_owned(),
                "ADMIN".to_owned(),
                "ANALYST".to_owned(),
            ],
            courier_policy: vec!["COURIER".to_owned(), "ADMIN".to_owned()],
            admin_policy: vec!["ADMIN".to_owned()],
            analyst_policy: vec!["ANALYST".to_owned()],
        }
    }
}