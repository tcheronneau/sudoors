use diesel::prelude::*;

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::common::schema::sudos)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Sudo {
    pub id: i32,
    pub username: String,
    pub hostnames: String,
}
impl Sudo {
    pub fn to_sudo(&self) -> crate::common::sudo::Sudo {
        crate::common::sudo::Sudo {
            username: self.username.clone(),
            duration: 0,
            hostnames: Some(self.hostnames.split(";").map(|s| s.to_string()).collect())
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::common::schema::sudos)]
pub struct NewSudo<'a> {
    pub username: &'a str,
    pub hostnames: &'a str,
}
