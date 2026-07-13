use crate::{Command, Manifest};
use schemars::Schema;

pub fn document_schema() -> Schema {
    schemars::schema_for!(Manifest)
}

pub fn command_schema() -> Schema {
    schemars::schema_for!(Vec<Command>)
}
