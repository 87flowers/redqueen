use argon2::{
    Argon2, PasswordHash,
    password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

#[derive(Clone, Debug)]
pub struct UserId(pub i64);

#[derive(Clone)]
pub struct Password(pub String);

impl Password {
    pub fn from_hash(hash: String) -> Self {
        Password(hash)
    }

    pub fn from_raw_password(password: &str) -> Option<Self> {
        let salt = SaltString::generate(&mut OsRng);
        Some(Password(Argon2::default().hash_password(password.as_bytes(), &salt).ok()?.to_string()))
    }

    pub fn matches_raw_password(self, password: &str) -> bool {
        let Ok(hash) = PasswordHash::new(&self.0) else {
            return false;
        };
        Argon2::default().verify_password(password.as_bytes(), &hash).is_ok()
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[...]")
    }
}

#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub password: Option<Password>,
    pub enabled: bool,
    pub admin: bool,
    pub autoapprove: bool,
    pub approver: bool,
}
