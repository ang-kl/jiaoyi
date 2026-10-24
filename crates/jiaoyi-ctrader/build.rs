//! Compiles Spotware's vendored .proto files (see NOTICE) with protox, a pure
//! Rust protobuf compiler, so no `protoc` binary is needed.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let files = [
        "proto/OpenApiCommonMessages.proto",
        "proto/OpenApiCommonModelMessages.proto",
        "proto/OpenApiMessages.proto",
        "proto/OpenApiModelMessages.proto",
    ];
    for f in files {
        println!("cargo:rerun-if-changed={f}");
    }
    let fds = protox::compile(files, ["proto"])?;
    prost_build::Config::new().compile_fds(fds)?;
    Ok(())
}
