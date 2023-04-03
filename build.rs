extern crate bindgen;
use bindgen::callbacks::{MacroParsingBehavior, ParseCallbacks};
use std::collections::HashSet;
use std::env;
use std::path::Path;

//, path::PathBuf};
//use glob::{glob};

const LIBRAW_DIR: &str = "libraw/";

const LIBRAW_FILES: [&str; 75] = [
    "libraw/src/decoders/canon_600.cpp",
    "libraw/src/decoders/crx.cpp",
    "libraw/src/decoders/decoders_dcraw.cpp",
    "libraw/src/decoders/decoders_libraw.cpp",
    "libraw/src/decoders/decoders_libraw_dcrdefs.cpp",
    "libraw/src/decoders/dng.cpp",
    "libraw/src/decoders/fp_dng.cpp",
    "libraw/src/decoders/fuji_compressed.cpp",
    "libraw/src/decoders/generic.cpp",
    "libraw/src/decoders/kodak_decoders.cpp",
    "libraw/src/decoders/load_mfbacks.cpp",
    "libraw/src/decoders/smal.cpp",
    "libraw/src/decoders/unpack.cpp",
    "libraw/src/decoders/unpack_thumb.cpp",
    "libraw/src/demosaic/aahd_demosaic.cpp",
    "libraw/src/demosaic/ahd_demosaic.cpp",
    "libraw/src/demosaic/dcb_demosaic.cpp",
    "libraw/src/demosaic/dht_demosaic.cpp",
    "libraw/src/demosaic/misc_demosaic.cpp",
    "libraw/src/demosaic/xtrans_demosaic.cpp",
    "libraw/src/integration/dngsdk_glue.cpp",
    "libraw/src/integration/rawspeed_glue.cpp",
    "libraw/src/metadata/adobepano.cpp",
    "libraw/src/metadata/canon.cpp",
    "libraw/src/metadata/ciff.cpp",
    "libraw/src/metadata/cr3_parser.cpp",
    "libraw/src/metadata/epson.cpp",
    "libraw/src/metadata/exif_gps.cpp",
    "libraw/src/metadata/fuji.cpp",
    "libraw/src/metadata/hasselblad_model.cpp",
    "libraw/src/metadata/identify.cpp",
    "libraw/src/metadata/identify_tools.cpp",
    "libraw/src/metadata/kodak.cpp",
    "libraw/src/metadata/leica.cpp",
    "libraw/src/metadata/makernotes.cpp",
    "libraw/src/metadata/mediumformat.cpp",
    "libraw/src/metadata/minolta.cpp",
    "libraw/src/metadata/misc_parsers.cpp",
    "libraw/src/metadata/nikon.cpp",
    "libraw/src/metadata/normalize_model.cpp",
    "libraw/src/metadata/olympus.cpp",
    "libraw/src/metadata/p1.cpp",
    "libraw/src/metadata/pentax.cpp",
    "libraw/src/metadata/samsung.cpp",
    "libraw/src/metadata/sony.cpp",
    "libraw/src/metadata/tiff.cpp",
    "libraw/src/postprocessing/aspect_ratio.cpp",
    "libraw/src/postprocessing/dcraw_process.cpp",
    "libraw/src/postprocessing/mem_image.cpp",
    "libraw/src/postprocessing/postprocessing_aux.cpp",
    "libraw/src/postprocessing/postprocessing_utils.cpp",
    "libraw/src/postprocessing/postprocessing_utils_dcrdefs.cpp",
    "libraw/src/preprocessing/ext_preprocess.cpp",
    "libraw/src/preprocessing/raw2image.cpp",
    "libraw/src/preprocessing/subtract_black.cpp",
    "libraw/src/tables/cameralist.cpp",
    "libraw/src/tables/colorconst.cpp",
    "libraw/src/tables/colordata.cpp",
    "libraw/src/tables/wblists.cpp",
    "libraw/src/utils/curves.cpp",
    "libraw/src/utils/decoder_info.cpp",
    "libraw/src/utils/init_close_utils.cpp",
    "libraw/src/utils/open.cpp",
    "libraw/src/utils/phaseone_processing.cpp",
    "libraw/src/utils/read_utils.cpp",
    "libraw/src/utils/thumb_utils.cpp",
    "libraw/src/utils/utils_dcraw.cpp",
    "libraw/src/utils/utils_libraw.cpp",
    "libraw/src/write/apply_profile.cpp",
    "libraw/src/write/file_write.cpp",
    "libraw/src/write/tiff_writer.cpp",
    "libraw/src/x3f/x3f_parse_process.cpp",
    "libraw/src/x3f/x3f_utils_patched.cpp",
    "libraw/src/libraw_c_api.cpp",
    "libraw/src/libraw_datastream.cpp",
];

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

    // add LIBRAW_FILES to libraw
    for file in LIBRAW_FILES.iter() {
        libraw.file(file);
    }

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
    println!("cargo:rustc-link-lib=c++");
    println!("cargo:rustc-link-lib=static=raw");
}

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
    bindgen::Builder::default()
        .use_core()
        .ctypes_prefix("cty")
        .generate_comments(true)
        .header("libraw/libraw/libraw.h")
        .parse_callbacks(Box::new(IgnoreMacros::new()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // API improvements
        .derive_eq(true)
        .size_t_is_usize(true)
        // these are never part of the API
        .blocklist_function("_.*")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
