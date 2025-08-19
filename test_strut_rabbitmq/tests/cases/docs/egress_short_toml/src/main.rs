use strut::RabbitMq;

#[strut::main]
async fn main() {
    let _publisher = RabbitMq::publisher("named_egress");
}
