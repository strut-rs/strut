use strut::AppProfile;

#[strut::main]
async fn main() {
    match AppProfile::active() {
        AppProfile::Prod => println!("We are in prod"),
        AppProfile::Dev => println!("We are in dev"),
        AppProfile::Test => println!("We are in test"),
        AppProfile::Custom(name) => {
            match name.as_str() {
                "preprod" => println!("We are in preprod"),
                other => println!("We are in {}", other),
            };
        }
    };
}
