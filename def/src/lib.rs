use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDefs(#[serde(with = "tuple_vec_map")] pub Vec<(String, TypeDef)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeDef(#[serde(with = "tuple_vec_map")] pub Vec<(String, FieldType)>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldType {
    Type(Type),
    Type2 {
        r#type: Type2,
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u32>,
        ref_type: String,
    },
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type {
    Byte,
    Byte4,
    Double,
    Dword,
    Filename,
    Float,
    Float2,
    Float3,
    Float4,
    Qword,
    #[serde(rename = "wchar *")]
    WcharRef,
    #[serde(rename = "char *")]
    CharRef,
    Word,
    Guid,
    Byte3,
    Dword2,
    Dword4,
    Word3,
    Fileref,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Type2 {
    Array,
    Ref,
    Struct,
}
