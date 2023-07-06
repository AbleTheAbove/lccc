use super::holeybytes::{HB_ARCH_PROPS, HB_PRIM_PROPS};
use crate::properties::LinkProperties;
use crate::properties::OperatingSystemProperties;
use crate::properties::TargetProperties;
use xlang_abi::const_sv;
use xlang_abi::span;
use xlang_abi::string::StringView;

pub static ABLEOS: OperatingSystemProperties = OperatingSystemProperties {
    is_unix_like: false,
    is_windows_like: false,
    os_family: span![const_sv!("ableos")],
    static_prefix: const_sv!("lib"),
    static_suffix: const_sv!(".a"),
    shared_prefix: const_sv!("lib"),
    shared_suffix: const_sv!(".so"),
    exec_suffix: StringView::empty(),
    obj_suffix: const_sv!(".o"),
    // TODO: subject to change
    ld_flavour: super::LinkerFlavor::Ld,
    ar_flavour: super::ArchiverFlavor::Ar,
    base_dirs: span![
        const_sv!("/System/"),
        // TODO: Implement wildcards
        // const_sv!("/Users/*/lib"),
    ],
    // TODO: subject to change
    so_kind: super::SharedLibraryStyle::Linkable,
};

pub static HB_ABLEOS_LINK: LinkProperties = LinkProperties {
    libdirs: span![const_sv!("lib")],
    // TODO
    default_libs: span![const_sv!("c")],
    // TODO
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    // TODO
    endfiles: span![const_sv!("crtn.o")],
    available_formats: span![],
    interp: const_sv!("ld-ableos.so"),
};
pub static HB_ABLEOS: TargetProperties = TargetProperties {
    primitives: &HB_PRIM_PROPS,
    os: &ABLEOS,
    arch: &HB_ARCH_PROPS,
    link: &HB_ABLEOS_LINK,
    abis: span![],
    enabled_features: span![],
};
