use std::{fs::File, path::Path};

use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};

use ascalon_ar_def::{FieldType, Type, Type2, TypeDefs};

pub fn generate(input_path: impl AsRef<Path>, output_path: impl AsRef<Path>) {
    let type_defs: TypeDefs = serde_json::from_reader(File::open(input_path).unwrap()).unwrap();
    let type_defs = type_defs.0.into_iter().map(|(type_name, type_def)| {
        let type_name = Ident::new(&type_name.to_case(Case::Pascal), Span::call_site());
        let fields = type_def.0.into_iter().map(|(field_name, field_type)| {
            let field_name = match field_name.as_str() {
                "type" => quote!(r#type),
                "move" => quote!(r#move),
                other => {
                    Ident::new(&other.to_case(Case::Snake), Span::call_site()).into_token_stream()
                }
            };
            let field_type = match field_type {
                FieldType::Type2 {
                    r#type,
                    size: _,
                    ref_type,
                } => {
                    let ref_type = match ref_type.as_str() {
                        "byte" => quote!(u8),
                        "byte4" => quote!([u8; 4]),
                        "double" => quote!(f64),
                        "dword" => quote!(u32),
                        "filename" => quote!(Filename),
                        "float" => quote!(f32),
                        "float2" => quote!([f32; 2]),
                        "float3" => quote!([f32; 3]),
                        "float4" => quote!([f32; 4]),
                        "qword" => quote!(u64),
                        "wchar *" => quote!(WcharRef),
                        "char *" => quote!(CharRef),
                        "word" => quote!(u16),
                        "guid" => quote!(Uuid),
                        "byte3" => quote!([u8; 3]),
                        "dword2" => quote!([u32; 2]),
                        "dword4" => quote!([u32; 4]),
                        "word3" => quote!([u16; 3]),
                        "fileref" => quote!(Fileref),
                        other => Ident::new(&other.to_case(Case::Pascal), Span::call_site())
                            .into_token_stream(),
                    };
                    match r#type {
                        Type2::Array => quote!(Array<#ref_type>),
                        Type2::Ref => quote!(Ref<#ref_type>),
                        Type2::Struct => quote!(#ref_type),
                    }
                }
                FieldType::Type(r#type) => match r#type {
                    Type::Byte => quote!(u8),
                    Type::Byte4 => quote!([u8; 4]),
                    Type::Double => quote!(f64),
                    Type::Dword => quote!(u32),
                    Type::Filename => quote!(Filename),
                    Type::Float => quote!(f32),
                    Type::Float2 => quote!([f32; 2]),
                    Type::Float3 => quote!([f32; 3]),
                    Type::Float4 => quote!([f32; 4]),
                    Type::Qword => quote!(u64),
                    Type::WcharRef => quote!(WcharRef),
                    Type::CharRef => quote!(CharRef),
                    Type::Word => quote!(u16),
                    Type::Guid => quote!(Uuid),
                    Type::Byte3 => quote!([u8; 3]),
                    Type::Dword2 => quote!([u32; 2]),
                    Type::Dword4 => quote!([u32; 4]),
                    Type::Word3 => quote!([u16; 3]),
                    Type::Fileref => quote!(Fileref),
                },
            };
            quote!(pub #field_name: #field_type)
        });
        quote! {
            pub struct #type_name {
                 #(#fields,)*
            }
        }
    });
    let type_defs = quote! {
         #(#type_defs)*
    };
    std::fs::write(output_path, type_defs.to_string()).unwrap();
}
