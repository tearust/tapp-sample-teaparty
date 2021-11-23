fn main() {
	prost_build::compile_protos(
		&["replica-provider.proto", "tokenstate-provider.proto"],
		&["../../tea-codec/proto"],
	)
	.unwrap();
}
