fn main() {
    prost_build::compile_protos(&["src/protos/model.proto", "src/protos/request.proto", "src/protos/response.proto", "src/protos/wrapper.proto"],
                                &["src/protos"]).unwrap();
}
