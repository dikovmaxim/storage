use crate::manager::Kvs::Kvs;

pub trait TableLookup {
    fn get_table_id_by_id(id: u128) -> String;
}