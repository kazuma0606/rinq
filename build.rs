fn main() {
    // protoファイルからRustコードを生成
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let hello_rs_path = std::path::Path::new(&out_dir).join("hello.rs");

    // Try to compile protos, if it fails create a stub
    if let Err(e) = prost_build::compile_protos(&["proto/hello.proto"], &["proto"]) {
        println!("cargo:warning=Skipping protobuf compilation: {}", e);

        // Create a stub hello.rs file
        std::fs::write(
            &hello_rs_path,
            r#"
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloRequest {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HelloResponse {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
"#,
        )
        .expect("Failed to write stub hello.rs");
    }

    // プロトファイルが変更されたときに再ビルドを促す
    println!("cargo:rerun-if-changed=proto/hello.proto");
}
