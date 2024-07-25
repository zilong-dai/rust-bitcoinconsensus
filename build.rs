extern crate cc;

use std::env;

fn main() {
    // Check whether we can use 64-bit compilation
    let use_64bit_compilation = if env::var("CARGO_CFG_TARGET_POINTER_WIDTH").unwrap() == "64" {
        let check = cc::Build::new()
            .file("depend/check_uint128_t.c")
            .cargo_metadata(false)
            .try_compile("check_uint128_t")
            .is_ok();
        if !check {
            println!("cargo:warning=Compiling in 32-bit mode on a 64-bit architecture due to lack of uint128_t support.");
        }
        check
    } else {
        false
    };
    let target = env::var("TARGET").expect("TARGET was not set");
    let is_big_endian = env::var("CARGO_CFG_TARGET_ENDIAN").expect("No endian is set") == "big";
    let mut base_config = cc::Build::new();
    base_config
        .include("depend/dogecoin/src/secp256k1/include")
        .define("__STDC_FORMAT_MACROS", None)
        .flag_if_supported("-Wno-implicit-fallthrough");

    if target.contains("windows") {
        base_config.define("WIN32", "1");
    }

    let mut secp_config = base_config.clone();
    let mut consensus_config = base_config;

    // **Secp256k1**
    if !cfg!(feature = "external-secp") {
        secp_config
            .include("depend/dogecoin/src/secp256k1/include")
            .include("depend/dogecoin/src/secp256k1/src")
            .flag_if_supported("-Wno-unused-function") // some ecmult stuff is defined but not used upstream
            .define("ECMULT_WINDOW_SIZE", "15")
            .define("ECMULT_GEN_PREC_BITS", "4")
            .define("ENABLE_MODULE_SCHNORRSIG", "1")
            .define("ENABLE_MODULE_EXTRAKEYS", "1")
            // Technically libconsensus doesn't require the ellswift and recovery features, but
            // `pubkey.cpp` does.
            .define("ENABLE_MODULE_ELLSWIFT", "1")
            .define("ENABLE_MODULE_RECOVERY", "1")
            .file("depend/dogecoin/src/secp256k1/src/precomputed_ecmult_gen.c")
            .file("depend/dogecoin/src/secp256k1/src/precomputed_ecmult.c")
            .file("depend/dogecoin/src/secp256k1/src/secp256k1.c");

        if is_big_endian {
            secp_config.define("WORDS_BIGENDIAN", "1");
        }

        if use_64bit_compilation {
            secp_config
                .define("USE_FIELD_5X52", "1")
                .define("USE_SCALAR_4X64", "1")
                .define("HAVE___INT128", "1");
        } else {
            secp_config.define("USE_FIELD_10X26", "1").define("USE_SCALAR_8X32", "1");
        }

        secp_config.compile("libsecp256k1.a");
    }

    let tool = consensus_config.get_compiler();
    if tool.is_like_msvc() {
        consensus_config.flag("/std:c++17").flag("/wd4100");
    } else if tool.is_like_clang() || tool.is_like_gnu() {
        consensus_config.flag("-std=c++17").flag("-Wno-unused-parameter");
    }

    consensus_config
        .cpp(true)
        .define("HAVE_CONFIG_H", "1")
        .include("depend/dogecoin/src")
        .include("depend/dogecoin/src/obj")
        .include("depend/dogecoin/src/secp256k1/include")
        .include("depend/dogecoin/src/consensus")
        .include("depend/dogecoin/src/crypto")
        .include("depend/dogecoin/src/primitives")
        .include("depend/dogecoin/src/script")
        .include("depend/dogecoin/src/config")

        .file("depend/dogecoin/src/crypto/aes.cpp")
        .file("depend/dogecoin/src/crypto/hmac_sha256.cpp")
        .file("depend/dogecoin/src/crypto/hmac_sha512.cpp")
        .file("depend/dogecoin/src/crypto/ripemd160.cpp")
        .file("depend/dogecoin/src/crypto/scrypt.cpp")
        .file("depend/dogecoin/src/crypto/sha1.cpp")
        .file("depend/dogecoin/src/crypto/sha256.cpp")
        .file("depend/dogecoin/src/crypto/sha512.cpp")

        .file("depend/dogecoin/src/arith_uint256.cpp")

        .file("depend/dogecoin/src/consensus/merkle.cpp")

        .file("depend/dogecoin/src/hash.cpp")

        .file("depend/dogecoin/src/primitives/block.cpp")
        .file("depend/dogecoin/src/primitives/pureheader.cpp")
        .file("depend/dogecoin/src/primitives/transaction.cpp")

        .file("depend/dogecoin/src/pubkey.cpp")

        .file("depend/dogecoin/src/script/bitcoinconsensus.cpp")
        .file("depend/dogecoin/src/script/interpreter.cpp")
        .file("depend/dogecoin/src/script/script.cpp")
        .file("depend/dogecoin/src/script/script_error.cpp")

        .file("depend/dogecoin/src/uint256.cpp")
        .file("depend/dogecoin/src/utilstrencodings.cpp")

        .compile("libdogecoinconsensus.a");
}
