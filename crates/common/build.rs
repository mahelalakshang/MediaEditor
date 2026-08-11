fn main() {
    let protoc_path = protoc_bin_vendored::protoc_bin_path()
        .expect("failed to locate vendored protoc binary");
    std::env::set_var("PROTOC", protoc_path);

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(&["../../proto/media.proto"], &["../../proto"])
        .expect("failed to compile proto/media.proto");
}
