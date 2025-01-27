use super::scope::Scope;

pub mod functions;
pub mod traits;
pub mod objects;

pub fn default_impl(s: &Scope) {
    traits::default_impl(s);
    functions::default_impl(s);
    objects::default_impl(s);
}
