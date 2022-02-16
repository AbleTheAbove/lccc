use target_tuples::ObjectFormat;
use xlang_abi::{const_sv, span, string::StringView};

use super::{LongDoubleFormat, OperatingSystemProperties, TargetProperties};

pub static LINUX: OperatingSystemProperties = OperatingSystemProperties {
    is_unix_like: true,
    is_windows_like: false,
    os_family: span![const_sv!("linux"), const_sv!("unix")],
    static_prefix: const_sv!("lib"),
    static_suffix: const_sv!(".a"),
    shared_prefix: const_sv!("lib"),
    shared_suffix: const_sv!(".so"),
    exec_suffix: StringView::empty(),
    obj_suffix: const_sv!(".o"),
    ld_flavour: super::LinkerFlavor::Ld,
    ar_flavour: super::ArchiverFlavor::Ar,
    base_dirs: span![const_sv!("/"), const_sv!("/usr"), const_sv!("/usr/local")],
    so_kind: super::SharedLibraryStyle::Linkable,
};

pub static X86_64_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 64,
    llongbits: 64,
    ptrbits: 64,
    max_align: 16,
    ptralign: 8,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xff,
    sizebits: 64,
    ldbl_align: 128,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib64")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x86-64.so.2"),
};

pub static X86_64_LINUX_GNUX32: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 16,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xff,
    sizebits: 64,
    ldbl_align: 128,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("libx32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x32.so.2"),
};

pub static X86_64V2_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 64,
    llongbits: 64,
    ptrbits: 64,
    max_align: 16,
    ptralign: 8,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V2,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib64")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x86-64.so.2"),
};

pub static X86_64V2_LINUX_GNUX32: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 16,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V2,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("libx32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x32.so.2"),
};

pub static X86_64V3_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 64,
    llongbits: 64,
    ptrbits: 64,
    max_align: 16,
    ptralign: 8,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V3,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib64")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x86-64.so.2"),
};

pub static X86_64V3_LINUX_GNUX32: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 16,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V3,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("libx32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x32.so.2"),
};

pub static X86_64V4_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 64,
    llongbits: 64,
    ptrbits: 64,
    max_align: 16,
    ptralign: 8,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V4,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib64")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x86-64.so.2"),
};

pub static X86_64V4_LINUX_GNUX32: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 16,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xffff,
    sizebits: 64,
    ldbl_align: 16,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::X86_64_V4,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("libx32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux-x32.so.2"),
};

pub static I386_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 32,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xf,
    sizebits: 32,
    ldbl_align: 4,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::I386,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux.so.2"),
};

pub static I486_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 32,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xf,
    sizebits: 32,
    ldbl_align: 4,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::I486,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux.so.2"),
};

pub static I586_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 32,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xf,
    sizebits: 32,
    ldbl_align: 4,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::I586,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux.so.2"),
};

pub static I686_LINUX_GNU: TargetProperties = TargetProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 32,
    max_align: 32,
    ptralign: 4,
    intmaxbits: 64,
    lock_free_atomic_mask: 0xf,
    sizebits: 32,
    ldbl_align: 4,
    ldbl_format: LongDoubleFormat::X87,
    arch: &super::x86::I686,
    os: &LINUX,
    libdirs: span![const_sv!("lib"), const_sv!("lib32")],
    default_libs: span![const_sv!("c")],
    startfiles: span![const_sv!("crt1.o"), const_sv!("crti.o")],
    endfiles: span![const_sv!("crtn.o")],
    enabled_features: span![],
    available_formats: span![ObjectFormat::Elf, ObjectFormat::Coff],
    interp: const_sv!("ld-linux.so.2"),
};
