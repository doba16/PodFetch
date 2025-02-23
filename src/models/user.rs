use std::io::Error;
use actix_web::HttpResponse;
use chrono::NaiveDateTime;
use diesel::prelude::{Insertable, Queryable};
use diesel::{OptionalExtension, RunQueryDsl, SqliteConnection, AsChangeset};
use diesel::associations::HasTable;
use utoipa::ToSchema;
use crate::schema::users;
use diesel::QueryDsl;
use diesel::ExpressionMethods;
use dotenv::var;
use crate::constants::constants::{BASIC_AUTH, OIDC_AUTH, Role, USERNAME};

#[derive(Serialize, Deserialize, Queryable, Insertable, Clone, ToSchema, PartialEq, Debug,
AsChangeset)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i32,
    pub username: String,
    pub role: String,
    pub password: Option<String>,
    pub explicit_consent: bool,
    pub created_at: NaiveDateTime
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWithoutPassword{
    pub id: i32,
    pub username: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub explicit_consent: bool
}


impl User{
    pub fn new(id: i32, username: String, role: Role, password: Option<String>, created_at:
    NaiveDateTime, explicit_consent: bool) -> Self {
        User {
            id,
            username,
            role: role.to_string(),
            password,
            created_at,
            explicit_consent
        }
    }

    pub fn find_by_username(username_to_find: &str, conn: &mut SqliteConnection) -> Option<User> {
        use crate::schema::users::dsl::*;

        match var(USERNAME) {
             Ok(res)=> {
                if res==username_to_find {
                    return Some(User::create_admin_user());
                }
            }
            _ => {}
        }

        users.filter(username.eq(username_to_find))
            .first::<User>(conn)
            .optional()
            .unwrap()
    }

    pub fn insert_user(&mut self, conn: &mut SqliteConnection) -> Result<User, Error> {
        use crate::schema::users::dsl::*;

        match  var(USERNAME){
            Ok(res) => {
                if res==self.username {
                    return Err(Error::new(std::io::ErrorKind::Other, "Username already exists"));
                }
            },
            Err(_) => {}
        }

        let res = diesel::insert_into(users::table())
            .values((
                username.eq(self.username.clone()),
                role.eq(self.role.clone()),
                password.eq(self.password.clone()),
                created_at.eq(chrono::Utc::now().naive_utc())
                ))
            .get_result::<User>(conn).unwrap();
        Ok(res)
    }

    pub fn delete_user(&self, conn: &mut SqliteConnection) -> Result<usize, diesel::result::Error> {
        diesel::delete(users::table.filter(users::id.eq(self.id)))
            .execute(conn)
    }

    pub fn update_role(&self, conn: &mut SqliteConnection) -> Result<UserWithoutPassword, diesel::result::Error> {
        let user = diesel::update(users::table.filter(users::id.eq(self.id)))
            .set(users::role.eq(self.role.clone()))
            .get_result::<User>(conn);

        Ok(User::map_to_dto(user.unwrap()))
    }

    fn create_admin_user()->User{
        User{
            id: 9999,
            username: var(USERNAME).unwrap(),
            role: Role::Admin.to_string(),
            password: None,
            explicit_consent: true,
            created_at: Default::default(),
        }
    }

    pub fn map_to_dto(user: Self) -> UserWithoutPassword{
        UserWithoutPassword{
            id: user.id,
            explicit_consent: user.explicit_consent,
            username: user.username.clone(),
            role: user.role.clone(),
            created_at: user.created_at
        }
    }

    pub fn find_all_users(conn: &mut SqliteConnection) -> Vec<UserWithoutPassword> {
        use crate::schema::users::dsl::*;

        let loaded_users = users.load::<User>(conn).unwrap();
        loaded_users.into_iter().map(|user| User::map_to_dto(user)).collect()
    }

    /**
        * Returns the username from the request header if the BASIC_AUTH environment variable is set to true
        * Otherwise returns None
     */
    pub fn get_username_from_req_header(req: &actix_web::HttpRequest) -> Result<Option<String>, Error>{
        if var(BASIC_AUTH).is_ok()|| var(OIDC_AUTH).is_ok() {
            let auth_header = req.headers().get(USERNAME);
            if auth_header.is_none() {
                return Err(Error::new(std::io::ErrorKind::Other, "Username not found"));
            }
            return Ok(Some(auth_header.unwrap().to_str().unwrap().parse().unwrap()))
        }
        Ok(None)
    }

    pub fn get_gpodder_req_header(req: &actix_web::HttpRequest) -> Result<String, Error>{
            let auth_header = req.headers().get(USERNAME);
            if auth_header.is_none() {
                return Err(Error::new(std::io::ErrorKind::Other, "Username not found"));
            }
            return Ok(auth_header.unwrap().to_str().unwrap().parse().unwrap())
    }


    pub fn check_if_admin_or_uploader(username: &Option<String>, conn: &mut SqliteConnection) ->
                                                                                              Option<HttpResponse> {
        if username.is_some(){
            let found_user = User::find_by_username(&username.clone().unwrap(), conn);
            if found_user.is_none(){
                return Some(HttpResponse::BadRequest().json("User not found"));
            }
            let user = found_user.unwrap();
            if user.role.ne(&Role::Admin.to_string()) && user.role.ne(&Role::Uploader.to_string()){
                return Some(HttpResponse::BadRequest().json("User is not an admin or uploader"));
            }
        }
        None
    }

    pub fn check_if_admin(username: &Option<String>, conn: &mut SqliteConnection) ->
                                                                                              Option<HttpResponse> {
        if username.is_some(){
            let found_user = User::find_by_username(&username.clone().unwrap(), conn);
            if found_user.is_none(){
                return Some(HttpResponse::BadRequest().json("User not found"));
            }
            let user = found_user.unwrap();

            if user.role != Role::Admin.to_string(){
                return Some(HttpResponse::BadRequest().json("User is not an admin"));
            }
        }
        None
    }

    pub fn delete_by_username(username_to_search: String, conn: &mut SqliteConnection)->Result<(), Error>{
        use crate::schema::users::dsl::*;
        diesel::delete(users.filter(username.eq(username_to_search))).execute(conn)
            .expect("Error deleting user");
        Ok(())
    }

    pub fn update_user(user: User, conn: &mut SqliteConnection)->Result<(), Error>{
        use crate::schema::users::dsl::*;
        diesel::update(users.filter(id.eq(user.clone().id)))
            .set(user).execute(conn)
            .expect("Error updating user");
        Ok(())
    }
}