use strut::AppConfig;

#[strut::main]
async fn main() {
    AppConfig::get()
        .database()
        .mysql_handles()
        .expect("MyDataBase");
    AppConfig::get()
        .database()
        .mysql_handles()
        .expect("OTHER_DATABASE");
}
