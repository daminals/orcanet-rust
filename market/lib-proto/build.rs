fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_build::compile_protos("../../proto/market.proto")?;
    tonic_build::configure()
        .type_attribute("User", "#[derive(Eq, serde::Deserialize, serde::Serialize)]")
        .compile(&["../../proto/market.proto"], &["../../proto/"])?;
    Ok(())
}
