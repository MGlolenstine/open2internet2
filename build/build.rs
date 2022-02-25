use protospec_build::NullImportResolver;

fn main() {
    protospec_build::compile_spec(
        "packets",
        include_str!("../spec/packets.pspec"),
        &protospec_build::Options {
            // include_async: true,
            use_anyhow: true,
            // debug_mode: true,
            ..Default::default()
        },
        NullImportResolver{}
    )
    .expect("failed to build packets.pspec");
}
