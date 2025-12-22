use syn::{Type, TypePath};

pub fn type_name(mut ty: &Type) -> Option<String> {
    loop {
        match ty {
            Type::Path(TypePath { qself: None, path }) => {
                return Some(path.segments.last().unwrap().ident.to_string());
            }
            Type::Group(group) => {
                ty = &group.elem;
            }
            _ => return None,
        }
    }
}
