fn main() -> Result<(), Box<dyn std::error::Error>> {
    //tonic_build::compile_protos("../../proto/market.proto")?;
    tonic_build::configure()
        .type_attribute("User", "#[derive(serde::Deserialize, serde::Serialize)]")
        .type_attribute(
            "FileInfo",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .type_attribute(
            "HoldersResponse",
            "#[derive(serde::Deserialize, serde::Serialize)]",
        )
        .compile(&["market.proto"], &["."])?;
    
    println!("cargo:rerun-if-changed=market.proto");
    println!("cargo:rerun-if-changed=.");
    Ok(())
}
