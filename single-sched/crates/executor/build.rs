fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tonic_build::compile_protos("../../proto/cmd.proto")?;
    tonic_build::configure()
        .include_file("mod.rs")
        .build_client(false)
        .build_server(true)
        .compile(&["../../proto/cmd.proto"], &["../../proto"])?;
    Ok(())
}
