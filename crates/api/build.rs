fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_prost_build::compile_protos("proto/system.proto")?;
    tonic_prost_build::compile_protos("proto/user.proto")?;

    println!("cargo:rerun-if-changed=proto");

    Ok(())
}
