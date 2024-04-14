fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "proto/market.proto";
    let coin_proto = "proto/coin.proto";

    tonic_build::configure()
        .type_attribute("User", "#[derive(serde::Deserialize, serde::Serialize)]")
        .compile(&[proto, coin_proto], &["../"])?;

    Ok(())
}
