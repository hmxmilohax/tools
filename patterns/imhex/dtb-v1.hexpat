#pragma endian little

import std.io;

struct String {
    u32 length [[color("C000C0")]];
    char text[length] [[color("FF6000")]];
} [[inline]];

struct Glob {
    u32 length [[color("C000C0")]];
    u8 data[length] [[color("FF6000")]];
} [[inline]];

enum DataType : u32 {
    Integer = 0,
    Real = 1,
    Variable = 2,
    Function = 3,
    Object = 4,
    Symbol = 5,
    Unhandled = 6,

    IfdefDirective = 7,
    ElseDirective = 8,
    EndifDirective = 9,

    Array = 16,
    Command = 17,
    String = 18,
    Property = 19,
    Glob = 20,

    DefineDirective = 32,
    IncludeDirective = 33,
    MergeDirective = 34,
    IfndefDirective = 35,
    AutorunDirective = 36,
    UndefineDirective = 37,
};

using DataArray;

struct DataNode {
    DataType type [[color("0080FF")]];
    match (type) {
        (DataType::Integer): s32 integer [[color("00FF00")]];
        (DataType::Real): float real [[color("00FF00")]];
        (DataType::Variable): String variable;
        (DataType::Function): String function;
        (DataType::Object): String object;
        (DataType::Symbol): String symbol;
        (DataType::Unhandled): s32 unhandled [[color("008000")]];

        (DataType::IfdefDirective): String ifdef_directive;
        (DataType::ElseDirective): s32 else_directive [[color("008000")]];
        (DataType::EndifDirective): s32 endif_directive [[color("008000")]];

        (DataType::Array): DataArray array [[inline]];
        (DataType::Command): DataArray command [[inline]];
        (DataType::String): String string;
        (DataType::Property): DataArray property [[inline]];
        (DataType::Glob): Glob glob;
        
        (DataType::DefineDirective): String define_directive;
        (DataType::IncludeDirective): String include_directive;
        (DataType::MergeDirective): String merge_directive;
        (DataType::IfndefDirective): String ifndef_directive;
        (DataType::AutorunDirective): s32 autorun_directive [[color("008000")]];
        (DataType::UndefineDirective): String undefine_directive;

        (_): {
            std::error(std::format("Invalid DataNode type value {}", type));
            return;
        }
    }
};

struct DataArray {
    u16 length [[color("C000C0")]];
    u16 id [[color("600080")]];
    u16 deprecated [[color("800060")]];
    DataNode nodes[length] [[inline]];
};

struct DataArrayFile {
    bool exists [[color("00FFFF")]];
    if (exists) {
        DataArray array [[inline]];
    }
};

DataArrayFile file @ 0 [[inline]];