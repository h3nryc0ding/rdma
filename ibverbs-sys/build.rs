use std::{env, path};

const CLANG_ARG: &str = "-I/usr/include";
const IBVERBS_LIB: &str = "libibverbs";
const IBVERBS_PKG: &str = "libibverbs-dev";
const IBVERBS_VERSION: &str = "1.14.41";

fn main() {
    pkg_config::Config::new()
        .atleast_version(IBVERBS_VERSION)
        .statik(false)
        .probe(IBVERBS_LIB)
        .unwrap_or_else(|_| {
            panic!(
                "Could not find {} version {}. Please install the {} package.",
                IBVERBS_LIB, IBVERBS_VERSION, IBVERBS_PKG
            );
        });

    let bindings = bindgen::Builder::default()
        .header_contents("include.h", "#include <infiniband/verbs.h>")
        .clang_arg(CLANG_ARG)
        .allowlist_function("ibv_.*")
        .allowlist_function("_ibv_.*")
        .allowlist_type("ibv_.*")
        .allowlist_type("verbs_context")
        .allowlist_var("IBV_LINK_LAYER_.*")
        .bitfield_enum("ibv_access_flags")
        .bitfield_enum("ibv_qp_attr_mask")
        .bitfield_enum("ibv_wc_flags")
        .bitfield_enum("ibv_send_flags")
        .bitfield_enum("ibv_port_cap_flags")
        .constified_enum_module("ibv_qp_type")
        .constified_enum_module("ibv_qp_state")
        .constified_enum_module("ibv_port_state")
        .constified_enum_module("ibv_wc_opcode")
        .constified_enum_module("ibv_wr_opcode")
        .constified_enum_module("ibv_wc_status")
        .derive_default(true)
        .derive_debug(true)
        .prepend_enum_name(false)
        .size_t_is_usize(true);

    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .generate()
        .expect("Unable to generate rdmacm")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write rdmacm!");
}
