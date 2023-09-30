use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef(#[serde(with = "tuple_vec_map")] pub Vec<(String, Type)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Type {
    RefType {
        r#type: TypeEnum,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u32>,
        ref_type: String,
    },
    Type(TypeEnum),
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TypeEnum {
    Array,
    Byte,
    Byte4,
    Double,
    Dword,
    Filename,
    Float,
    Float2,
    Float3,
    Float4,
    Ptr,
    Qword,
    #[serde(rename = "wchar *")]
    WcharPtr,
    #[serde(rename = "char *")]
    CharPtr,
    Struct,
    Word,
    Guid,
    Byte3,
    Dword2,
    Dword4,
    Word3,
    Fileref,
}
