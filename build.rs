extern crate bindgen;
use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::collections::HashSet;
use std::env;
use std::path::Path;

//, path::PathBuf};
//use glob::{glob};

const LIBRAW_DIR: &str = "libraw/";

fn main() {
    let out_dir_ = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir_);

    build(out_dir);
    bindings(out_dir);
}

fn build(out_dir: &Path) {
    let mut libraw = cc::Build::new();
    libraw.cpp(true);
    libraw.include(LIBRAW_DIR);

    libraw.file("libraw/src/decoders/canon_600.cpp");
    libraw.file("libraw/src/decoders/crx.cpp");
    libraw.file("libraw/src/decoders/decoders_dcraw.cpp");
    libraw.file("libraw/src/decoders/decoders_libraw.cpp");
    libraw.file("libraw/src/decoders/decoders_libraw_dcrdefs.cpp");
    libraw.file("libraw/src/decoders/dng.cpp");
    libraw.file("libraw/src/decoders/fp_dng.cpp");
    libraw.file("libraw/src/decoders/fuji_compressed.cpp");
    libraw.file("libraw/src/decoders/generic.cpp");
    libraw.file("libraw/src/decoders/kodak_decoders.cpp");
    libraw.file("libraw/src/decoders/load_mfbacks.cpp");
    libraw.file("libraw/src/decoders/smal.cpp");
    libraw.file("libraw/src/decoders/unpack.cpp");
    libraw.file("libraw/src/decoders/unpack_thumb.cpp");
    libraw.file("libraw/src/demosaic/aahd_demosaic.cpp");
    libraw.file("libraw/src/demosaic/ahd_demosaic.cpp");
    libraw.file("libraw/src/demosaic/dcb_demosaic.cpp");
    libraw.file("libraw/src/demosaic/dht_demosaic.cpp");
    libraw.file("libraw/src/demosaic/misc_demosaic.cpp");
    libraw.file("libraw/src/demosaic/xtrans_demosaic.cpp");
    libraw.file("libraw/src/integration/dngsdk_glue.cpp");
    libraw.file("libraw/src/integration/rawspeed_glue.cpp");
    libraw.file("libraw/src/metadata/adobepano.cpp");
    libraw.file("libraw/src/metadata/canon.cpp");
    libraw.file("libraw/src/metadata/ciff.cpp");
    libraw.file("libraw/src/metadata/cr3_parser.cpp");
    libraw.file("libraw/src/metadata/epson.cpp");
    libraw.file("libraw/src/metadata/exif_gps.cpp");
    libraw.file("libraw/src/metadata/fuji.cpp");
    libraw.file("libraw/src/metadata/hasselblad_model.cpp");
    libraw.file("libraw/src/metadata/identify.cpp");
    libraw.file("libraw/src/metadata/identify_tools.cpp");
    libraw.file("libraw/src/metadata/kodak.cpp");
    libraw.file("libraw/src/metadata/leica.cpp");
    libraw.file("libraw/src/metadata/makernotes.cpp");
    libraw.file("libraw/src/metadata/mediumformat.cpp");
    libraw.file("libraw/src/metadata/minolta.cpp");
    libraw.file("libraw/src/metadata/misc_parsers.cpp");
    libraw.file("libraw/src/metadata/nikon.cpp");
    libraw.file("libraw/src/metadata/normalize_model.cpp");
    libraw.file("libraw/src/metadata/olympus.cpp");
    libraw.file("libraw/src/metadata/p1.cpp");
    libraw.file("libraw/src/metadata/pentax.cpp");
    libraw.file("libraw/src/metadata/samsung.cpp");
    libraw.file("libraw/src/metadata/sony.cpp");
    libraw.file("libraw/src/metadata/tiff.cpp");
    libraw.file("libraw/src/postprocessing/aspect_ratio.cpp");
    libraw.file("libraw/src/postprocessing/dcraw_process.cpp");
    libraw.file("libraw/src/postprocessing/mem_image.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_aux.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_utils.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_utils_dcrdefs.cpp");
    libraw.file("libraw/src/preprocessing/ext_preprocess.cpp");
    libraw.file("libraw/src/preprocessing/raw2image.cpp");
    libraw.file("libraw/src/preprocessing/subtract_black.cpp");
    libraw.file("libraw/src/tables/cameralist.cpp");
    libraw.file("libraw/src/tables/colorconst.cpp");
    libraw.file("libraw/src/tables/colordata.cpp");
    libraw.file("libraw/src/tables/wblists.cpp");
    libraw.file("libraw/src/utils/curves.cpp");
    libraw.file("libraw/src/utils/decoder_info.cpp");
    libraw.file("libraw/src/utils/init_close_utils.cpp");
    libraw.file("libraw/src/utils/open.cpp");
    libraw.file("libraw/src/utils/phaseone_processing.cpp");
    libraw.file("libraw/src/utils/read_utils.cpp");
    libraw.file("libraw/src/utils/thumb_utils.cpp");
    libraw.file("libraw/src/utils/utils_dcraw.cpp");
    libraw.file("libraw/src/utils/utils_libraw.cpp");
    libraw.file("libraw/src/write/apply_profile.cpp");
    libraw.file("libraw/src/write/file_write.cpp");
    libraw.file("libraw/src/write/tiff_writer.cpp");
    libraw.file("libraw/src/x3f/x3f_parse_process.cpp");
    libraw.file("libraw/src/x3f/x3f_utils_patched.cpp");
    libraw.file("libraw/src/libraw_c_api.cpp");
    libraw.file("libraw/src/libraw_datastream.cpp");

    libraw.warnings(false);
    libraw.extra_warnings(false);
    // do I really have to supress all of these?
    libraw.flag_if_supported("-Wno-format-truncation");
    libraw.flag_if_supported("-Wno-unused-result");
    libraw.flag_if_supported("-Wno-format-overflow");
    // thread safety
    libraw.flag("-Wno-deprecated-declarations");
    libraw.flag("-pthread");
    libraw.static_flag(true);
    libraw.compile("raw");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=raw");
}

const IGNORE_MACROS: [&str; 20] = [
    "FE_DIVBYZERO",
    "FE_DOWNWARD",
    "FE_INEXACT",
    "FE_INVALID",
    "FE_OVERFLOW",
    "FE_TONEAREST",
    "FE_TOWARDZERO",
    "FE_UNDERFLOW",
    "FE_UPWARD",
    "FP_INFINITE",
    "FP_INT_DOWNWARD",
    "FP_INT_TONEAREST",
    "FP_INT_TONEARESTFROMZERO",
    "FP_INT_TOWARDZERO",
    "FP_INT_UPWARD",
    "FP_NAN",
    "FP_NORMAL",
    "FP_SUBNORMAL",
    "FP_ZERO",
    "IPPORT_RESERVED",
];

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> MacroParsingBehavior {
        if self.0.contains(name) {
            MacroParsingBehavior::Ignore
        } else {
            MacroParsingBehavior::Default
        }
    }
}

impl IgnoreMacros {
    fn new() -> Self {
        Self(IGNORE_MACROS.into_iter().map(|s| s.to_owned()).collect())
    }
}

fn bindings(out_dir: &Path) {

    //detect current platform
    let target = env::var("TARGET").unwrap();

    let mut bindings = bindgen::Builder::default()        
        .use_core()
        .generate_comments(true);

    if target.contains("linux") {
        bindings = bindings.ctypes_prefix("libc");
    }
    
    let builder = bindings.header("libraw/libraw/libraw.h")
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // API improvements
        .derive_eq(true)
        .size_t_is_usize(true)
        // these are never part of the API
        .blocklist_function("_.*")
        // Rust doesn't support long double, and bindgen can't skip it
        // https://github.com/rust-lang/rust-bindgen/issues/1549
        .blocklist_function("acoshl")
        .blocklist_function("acosl")
        .blocklist_function("asinhl")
        .blocklist_function("asinl")
        .blocklist_function("atan2l")
        .blocklist_function("atanhl")
        .blocklist_function("atanl")
        .blocklist_function("cbrtl")
        .blocklist_function("ceill")
        .blocklist_function("copysignl")
        .blocklist_function("coshl")
        .blocklist_function("cosl")
        .blocklist_function("dreml")
        .blocklist_function("ecvt_r")
        .blocklist_function("erfcl")
        .blocklist_function("erfl")
        .blocklist_function("exp2l")
        .blocklist_function("expl")
        .blocklist_function("expm1l")
        .blocklist_function("fabsl")
        .blocklist_function("fcvt_r")
        .blocklist_function("fdiml")
        .blocklist_function("finitel")
        .blocklist_function("floorl")
        .blocklist_function("fmal")
        .blocklist_function("fmaxl")
        .blocklist_function("fminl")
        .blocklist_function("fmodl")
        .blocklist_function("frexpl")
        .blocklist_function("gammal")
        .blocklist_function("hypotl")
        .blocklist_function("ilogbl")
        .blocklist_function("isinfl")
        .blocklist_function("isnanl")
        .blocklist_function("j0l")
        .blocklist_function("j1l")
        .blocklist_function("jnl")
        .blocklist_function("ldexpl")
        .blocklist_function("lgammal")
        .blocklist_function("lgammal_r")
        .blocklist_function("llrintl")
        .blocklist_function("llroundl")
        .blocklist_function("log10l")
        .blocklist_function("log1pl")
        .blocklist_function("log2l")
        .blocklist_function("logbl")
        .blocklist_function("logl")
        .blocklist_function("lrintl")
        .blocklist_function("lroundl")
        .blocklist_function("modfl")
        .blocklist_function("nanl")
        .blocklist_function("nearbyintl")
        .blocklist_function("nextafterl")
        .blocklist_function("nexttoward")
        .blocklist_function("nexttowardf")
        .blocklist_function("nexttowardl")
        .blocklist_function("powl")
        .blocklist_function("qecvt")
        .blocklist_function("qecvt_r")
        .blocklist_function("qfcvt")
        .blocklist_function("qfcvt_r")
        .blocklist_function("qgcvt")
        .blocklist_function("remainderl")
        .blocklist_function("remquol")
        .blocklist_function("rintl")
        .blocklist_function("roundl")
        .blocklist_function("scalbl")
        .blocklist_function("scalblnl")
        .blocklist_function("scalbnl")
        .blocklist_function("significandl")
        .blocklist_function("sinhl")
        .blocklist_function("sinl")
        .blocklist_function("sqrtl")
        .blocklist_function("strtold")
        .blocklist_function("tanhl")
        .blocklist_function("tanl")
        .blocklist_function("tgammal")
        .blocklist_function("truncl")
        .blocklist_function("y0l")
        .blocklist_function("y1l")
        .blocklist_function("ynl")
        .generate()
        .expect("Unable to generate bindings");

    builder
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
