use super::MachineProperties;
use crate::properties::asm::AsmScalar;
use crate::properties::ArchProperties;
use crate::properties::AsmProperties;
use crate::properties::PrimitiveProperties;
use crate::properties::Span;
use xlang_abi::const_sv;
use xlang_abi::string::StringView;
use xlang_abi::{pair::Pair, span};

pub static HB_ARCH_PROPS: ArchProperties = ArchProperties {
    lock_free_atomic_masks: 0x00,
    builtins: span![],
    target_features: span![],
    machines: span![Pair(const_sv!("generic"), &HB_MACHINE_PROPS)],
    default_machine: &HB_MACHINE_PROPS,
    arch_names: span![const_sv!("holeybytes")],
    byte_order: super::ByteOrder::LittleEndian,
    asm_propreties: &HB_ASM_PROPS,
};

pub static HB_PRIM_PROPS: PrimitiveProperties = PrimitiveProperties {
    intbits: 32,
    longbits: 32,
    llongbits: 64,
    ptrbits: 64,
    fnptrbits: 64,
    nearptrbits: 64,
    farptrbits: 64,
    max_align: 16, // Ping ondra and make him decide this
    ptralign: 8,
    intmaxbits: 64,
    sizebits: 64,
    lock_free_atomic_mask: 0x00,
    ldbl_align: 8,
    ldbl_format: super::LongDoubleFormat::IEEE64,
    max_atomic_align: 16,
};

static HB_MACHINE_PROPS: MachineProperties = MachineProperties {
    default_features: span![],
};

static HB_ASM_PROPS: AsmProperties = AsmProperties {
    syntax_names: span![const_sv!("standard")],
    constraints: HB_ASM_CONSTRAINTS,
    register_groups: HB_REGISTER_GROUPS,
    overlaps: HB_ASM_REGISTER_OVERLAPS,
    classes: HB_CLASSES,
};

const HB_REGISTER_GROUPS: Span<
    'static,
    Pair<StringView<'static>, Span<'static, StringView<'static>>>,
> = span![];
const HB_CLASSES: Span<'static, Pair<StringView<'static>, StringView<'static>>> = span![];
const HB_ASM_REGISTER_OVERLAPS: Span<'static, Pair<StringView<'static>, StringView<'static>>> =
    span![];

const HB_ASM_CONSTRAINTS: Span<'static, Pair<StringView<'static>, AsmScalar>> = span![];
