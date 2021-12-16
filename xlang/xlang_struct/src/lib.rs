#![deny(warnings, clippy::all, clippy::pedantic, clippy::nursery)]
pub use xlang_abi::prelude::v1::*;
use xlang_targets::Target;

#[repr(u16)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum PathComponent {
    Root,
    Text(String),
    SpecialComponent(String),
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Path {
    pub components: Vec<PathComponent>,
}

#[repr(u8)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum AnnotationItem {
    Meta(Box<Annotation>),
    Identifier(Path),
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct Annotation {
    pub items: Vec<AnnotationItem>,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq, Default)]
pub struct AnnotatedElement {
    pub annotations: Vec<Annotation>,
}

#[repr(u16)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Visibility {
    Public = 0,
    Origin = 1,
    Module = 2,
    Private = 3,
    None = 4,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Public
    }
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct ScopeMember {
    pub annotations: AnnotatedElement,
    pub vis: Visibility,
    pub member_decl: MemberDeclaration,
}

#[repr(C)]
#[derive(Clone, Debug, Default)]
pub struct Scope {
    pub annotations: AnnotatedElement,
    pub members: HashMap<Path, ScopeMember>,
}

#[repr(u32)]
#[derive(Clone, Debug)]
pub enum MemberDeclaration {
    Empty,
    Scope(Scope),
    Function(FunctionDeclaration),
}

impl Default for MemberDeclaration {
    fn default() -> Self {
        Self::Empty
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Default)]
    pub struct ScalarValidity : u8{
        const NONZERO = 0x01;
        const FINITE = 0x02;
        const NONNAN = 0x04;
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ScalarTypeHeader {
    pub bitsize: u16,
    pub vectorsize: u16,
    pub validity: ScalarValidity,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
#[repr(u8)]
pub enum ScalarTypeKind {
    Empty,
    Integer { signed: bool, min: i128, max: i128 },
    Float { decimal: bool },
    Fixed { fractbits: u16 },
}

impl Default for ScalarTypeKind {
    fn default() -> Self {
        Self::Empty
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ScalarType {
    pub header: ScalarTypeHeader,
    pub kind: ScalarTypeKind,
}

#[repr(u16)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    Scalar(ScalarType),
    Void,
    FnType(Box<FnType>),
    Pointer(PointerType),
}

fake_enum::fake_enum! {
    #[repr(u8)]
    #[derive(Default,Hash)]
    pub enum struct PointerAliasingRule{
        None = 0,
        Unique = 1,
        ReadOnly = 2,
        ReadShallow = 3,
        Invalid = 4,
        Nonnull = 5,
        Volatile = 6,
        VolatileWrite = 7,
        NullOrInvalid = 8,
    }
}

fake_enum::fake_enum! {
    #[repr(u8)]
    #[derive(Default,Hash)]
    pub enum struct ValidRangeType{
        None = 0,
        Dereference = 1,
        DereferenceWrite = 2,
        WriteOnly = 3,
        NullOrDereference = 4,
        NullOrDereferenceWrite = 5,
        NullOrWriteOnly = 6,
    }
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PointerType {
    pub alias: PointerAliasingRule,
    pub valid_range: Pair<ValidRangeType, u64>,
    pub inner: Box<Type>,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FnType {
    pub ret: Type,
    pub params: Vec<Type>,
    pub tag: Abi,
}

#[repr(u16)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    Invalid,
    Uninitialized,
    GenericParameter(u32),
    Integer { ty: ScalarType, val: u32 },
    GlobalAddress { ty: Type, item: Path },
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
    BitXor,
    Rsh,
    Lsh,

    CmpInt,
    CmpLt,
    CmpGt,
    CmpEq,
    CmpNe,
    CmpGe,
    GmpLe,
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Minus,
    BitNot,
    LogicNot,
}

#[repr(u16)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Expr {
    Null,
    Const(Value),
    ExitBlock { blk: u32, values: u16 },
    BinaryOp(BinaryOp),
    UnaryOp(UnaryOp),
    CallFunction(FnType),
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum StackValueKind {
    LValue,
    RValue,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct StackItem {
    pub ty: Type,
    pub kind: StackValueKind,
}

#[repr(u8)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum BlockItem {
    Expr(Expr),
    Target { num: u32, stack: Vec<StackItem> },
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[repr(u16)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Abi {
    C,
    Cdecl,
    Fastcall,
    Stdcall,
    Vectorcall,
    Thiscall,
    SysV,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FunctionDeclaration {
    pub ty: FnType,
    pub body: Option<Block>,
}

#[repr(C)]
#[derive(Clone, Debug, Hash, PartialEq)]
pub struct File {
    pub target: Target,
    pub root: Block,
}
