pub mod home;
pub mod login;

#[derive(Debug, Clone, Copy, Default)]
pub enum Page {
    #[default]
    Home,
    Login,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/login",
        }
    }
}
