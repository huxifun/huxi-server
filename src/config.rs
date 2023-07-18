use serde::{Deserialize, Serialize};

#[derive(clap::Parser, Clone)]
pub struct WebArgs {
    #[clap(long, env)]
    pub www_config: String,
    #[clap(long, env)]
    pub www_port: u16,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Host {
    pub name: String,
    pub www: String,
    pub my_name: String,
    pub logo: String,
    pub icp: String,
    pub copyright: String,
    pub domain: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Database {
    pub url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EmailServer {
    pub stmp_server: String,
    pub stmp_user: String,
    pub stmp_password: String,
    pub stmp_from: String,
}

pub trait CategoryType {
    fn name(&self, path: &str) -> Option<String>;
    fn path_name(&self, id: u8) -> Option<(String, String)>;
    fn id(&self, path: &str) -> Option<u8>;
}

impl CategoryType for Vec<(u8, String, String)> {
    fn name(&self, path: &str) -> Option<String> {
        self.iter().find(|x| x.1 == path).map(|x| x.2.clone())
    }
    fn path_name(&self, id: u8) -> Option<(String, String)> {
        self.iter()
            .find(|x| x.0 == id)
            .map(|x| (x.1.clone(), x.2.clone()))
    }
    fn id(&self, path: &str) -> Option<u8> {
        self.iter().find(|x| x.1 == path).map(|x| x.0)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Hu {
    pub content_type: Vec<(u8, String, String)>,
    pub category: Vec<(u8, String, String)>,
    pub page_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Book {
    pub content_type: Vec<(u8, String, String)>,
    pub category: Vec<(u8, String, String)>,
    pub upload_path: String,
    pub public_url: String,
    pub page_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Xi {
    pub content_type: Vec<(u8, String, String)>,
    pub category: Vec<(u8, String, String)>,
    pub page_size: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Image {
    pub upload_path: String,
    pub public_url: String,
    pub resize: Vec<(String, u32)>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct User {
    pub upload_path: String,
    pub public_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebConfig {
    pub host: Host,
    pub database: Database,
    pub email: EmailServer,
    pub hu: Hu,
    pub book: Book,
    pub xi: Xi,
    pub image: Image,
    pub user: User,
}
