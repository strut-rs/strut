use strut::RabbitMq;

#[strut::main]
async fn main() {
    let _subscriber = RabbitMq::undecoded_subscriber("named_ingress");
}
