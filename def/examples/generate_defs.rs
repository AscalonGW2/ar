use std::fs::File;

use byteorder::{ReadBytesExt, LE};
use object::{pe::ImageSectionHeader, read::pe::PeFile64, Object};

use ascalon_ar_def::{Type, TypeDef, TypeDefs, TypeEnum};

fn main() {
    let mut image = std::fs::read(r#"C:\Program Files\Guild Wars 2\Gw2-64.exe"#).unwrap();
    let image = PeFile64::parse(image.as_slice()).unwrap();

    let mut type_defs = Vec::new();

    // Search the whole .rdata section
    let base = image.relative_address_base();
    let section = image
        .section_table()
        .iter()
        .find(|section| section.name.as_slice() == b".rdata\0\0")
        .unwrap();
    let mut data = section.pe_data(image.data()).unwrap();
    loop {
        if data.len() < 4 + 4 + 8 {
            break;
        }

        // Pack chunk type has always 4 alphabetic characters
        let (name, mut test_data) = data.split_at(4);
        if !name.iter().all(|c| c.is_ascii_alphabetic()) {
            data = &data[1..];
            continue;
        }

        // Type def count is never zero
        let type_def_count = test_data.read_u32::<LE>().unwrap();
        if type_def_count == 0 {
            data = &data[1..];
            continue;
        }

        // Type defs rva has to be within the .rdata section
        let type_defs_rva = (test_data.read_u64::<LE>().unwrap() - base) as u32;
        let Some(mut type_defs_data) = section.pe_data_at(image.data(), type_defs_rva) else {
            data = &data[1..];
            continue;
        };
        if type_defs_data.len() < (type_def_count * (8 + 8 + 8)) as usize {
            data = &data[1..];
            continue;
        }

        // Verify that all type defs are valid
        let mut test_type_defs_data = type_defs_data;
        if (0..type_def_count).any(|_| {
            let type_def_rva = (test_type_defs_data.read_u64::<LE>().unwrap() - base) as u32;
            let _1 = test_type_defs_data.read_u64::<LE>().unwrap();
            let _2 = test_type_defs_data.read_u64::<LE>().unwrap();
            !section.contains_rva(type_def_rva) || _1 != 0 || _2 != 0
        }) {
            data = &data[1..];
            continue;
        }

        println!("{}", std::str::from_utf8(name).unwrap());
        for _ in 0..type_def_count {
            let type_def_rva = (type_defs_data.read_u64::<LE>().unwrap() - base) as u32;
            let _1 = type_defs_data.read_u64::<LE>().unwrap();
            let _2 = type_defs_data.read_u64::<LE>().unwrap();
            generate_type_def(base, image.data(), section, type_def_rva, &mut type_defs);
        }

        data = &data[4 + 4 + 8..];
    }

    serde_json::to_writer_pretty(File::create("packfile.json").unwrap(), &TypeDefs(type_defs))
        .unwrap();
}

fn generate_type_def(
    base: u64,
    image: &[u8],
    section: &ImageSectionHeader,
    rva: u32,
    type_defs: &mut Vec<(String, TypeDef)>,
) -> String {
    let mut data = section.pe_data_at(image, rva).unwrap();
    let mut fields = vec![];
    let name = loop {
        let r#type = data.read_u16::<LE>().unwrap();
        let _1 = data.read_u16::<LE>().unwrap();
        let _2 = data.read_u32::<LE>().unwrap();
        let name_rva = (data.read_u64::<LE>().unwrap() - base) as u32;
        let type_def_rva = (data.read_u64::<LE>().unwrap() - base) as u32;
        let size = data.read_u64::<LE>().unwrap();

        let name: String = section
            .pe_data_at(image, name_rva)
            .unwrap()
            .iter()
            .map_while(|&c| if c != 0 { Some(char::from(c)) } else { None })
            .collect();
        if r#type == 0 {
            break name;
        }

        fields.push((
            name.clone(),
            match r#type {
                1 => Type::RefType {
                    r#type: TypeEnum::Array,
                    size: Some(size as u32),
                    ref_type: generate_type_def(base, image, section, type_def_rva, type_defs),
                },
                2 => Type::RefType {
                    r#type: TypeEnum::Array,
                    size: None,
                    ref_type: generate_type_def(base, image, section, type_def_rva, type_defs),
                },
                5 => Type::Type(TypeEnum::Byte),
                6 => Type::Type(TypeEnum::Byte4),
                7 => Type::Type(TypeEnum::Double),
                10 => Type::Type(TypeEnum::Dword),
                11 => Type::Type(TypeEnum::Filename),
                12 => Type::Type(TypeEnum::Float),
                13 => Type::Type(TypeEnum::Float2),
                14 => Type::Type(TypeEnum::Float3),
                15 => Type::Type(TypeEnum::Float4),
                16 => Type::RefType {
                    r#type: TypeEnum::Ref,
                    size: None,
                    ref_type: generate_type_def(base, image, section, type_def_rva, type_defs),
                },
                17 => Type::Type(TypeEnum::Qword),
                18 => Type::Type(TypeEnum::WcharRef),
                19 => Type::Type(TypeEnum::CharRef),
                20 => Type::RefType {
                    r#type: TypeEnum::Struct,
                    size: None,
                    ref_type: generate_type_def(base, image, section, type_def_rva, type_defs),
                },
                21 => Type::Type(TypeEnum::Word),
                22 => Type::Type(TypeEnum::Guid),
                23 => Type::Type(TypeEnum::Byte3),
                24 => Type::Type(TypeEnum::Dword2),
                25 => Type::Type(TypeEnum::Dword4),
                26 => Type::Type(TypeEnum::Word3),
                27 => Type::Type(TypeEnum::Fileref),
                28 => Type::RefType {
                    r#type: TypeEnum::Array,
                    size: Some(size as u32),
                    ref_type: "byte".to_string(),
                },
                _ => todo!(),
            },
        ));
    };

    // Slow but the order is preserved
    if type_defs.iter().all(|type_def| type_def.0 != name) {
        type_defs.push((name.clone(), TypeDef(fields)));
    }

    name
}
