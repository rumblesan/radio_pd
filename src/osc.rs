use libpd_rs::types::Atom;
use rosc::OscType;

pub fn osc_type_to_atom(osc_type: OscType) -> Option<Atom> {
    match osc_type {
        OscType::Int(v) => Some(Atom::Float(v.into())),
        OscType::Float(v) => Some(Atom::Float(v.into())),
        OscType::String(v) => Some(Atom::Symbol(v.into())),
        OscType::Blob(..) => None,
        OscType::Time(..) => None,
        OscType::Long(..) => None,
        OscType::Double(v) => Some(Atom::Float(v.into())),
        _ => None,
    }
}
