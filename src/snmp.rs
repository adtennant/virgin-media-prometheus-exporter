use anyhow::{anyhow, Context, Error, Result};
use std::{
    borrow::Cow,
    collections::{hash_map::Iter, HashMap},
    convert::TryFrom,
    fmt::{Debug, Display},
    str::FromStr,
};

const SCALAR_SUFFIX: &'static str = ".0";
const TABLE_ENTRY_SUFFIX: &'static str = ".1";

#[derive(Clone, Debug, Eq, Hash, PartialEq, serde::Deserialize)]
pub struct OID(Cow<'static, str>);

impl OID {
    pub const fn new(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    fn trim_suffix(self, s: &str) -> OID {
        if self.0.as_ref().ends_with(s) {
            OID(Cow::Owned(
                self.0.as_ref()[..self.0.as_ref().len() - s.len()].to_owned(),
            ))
        } else {
            self
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct List(HashMap<OID, String>);

#[derive(Clone, Debug)]
pub struct TableEntry(HashMap<OID, String>);

impl TableEntry {
    pub fn get_column(&self, oid: &OID) -> Result<&String> {
        self.0
            .get(oid)
            .context(format!("column not found: {}", oid.0))
    }

    pub fn parse_column<T>(&self, oid: &OID) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        self.get_column(oid)
            .and_then(|s| T::from_str(s).map_err(|e| anyhow!("{:?}", e)))
    }
}

#[derive(Clone, Debug)]
pub struct Table<T = TableEntry>(HashMap<String, T>);

impl<T> Table<T> {
    pub fn get(&self, index: &str) -> Option<&T> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter<'_, String, T> {
        self.0.iter()
    }
}

impl List {
    pub fn get_scalar(&self, oid: &OID) -> Result<&String> {
        let oid = OID(Cow::Owned(format!("{}{}", oid.0, SCALAR_SUFFIX)));
        self.0
            .get(&oid)
            .context(format!("scalar not found: {}", oid.0))
    }

    pub fn parse_scalar<T>(&self, oid: &OID) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: Debug + Display + Send + Sync + 'static,
    {
        self.get_scalar(oid)
            .and_then(|s| T::from_str(s).map_err(Error::msg))
    }

    pub fn get_table(&self, oid: &OID) -> Result<Table> {
        let mut table: HashMap<String, TableEntry> = HashMap::new();
        let table_entry_prefix = format!("{}{}.", oid.0, TABLE_ENTRY_SUFFIX);

        for (oid, value) in self
            .0
            .iter()
            .filter(|(oid, _)| oid.0.starts_with(&table_entry_prefix))
        {
            let index = oid
                .0
                .trim_start_matches(&table_entry_prefix)
                .splitn(2, ".")
                .nth(1)
                .context("failed to extract table index")?
                .to_owned();

            let column = oid.clone().trim_suffix(&format!(".{}", index));

            let entry = table.entry(index).or_insert(TableEntry(HashMap::new()));
            entry.0.insert(column, value.clone());
        }

        Ok(Table(table))
    }

    pub fn parse_table<T>(&self, oid: &OID) -> Result<Table<T>>
    where
        T: TryFrom<TableEntry>,
        <T as TryFrom<TableEntry>>::Error: Debug + Display + Send + Sync + 'static,
    {
        self.get_table(oid)
            .and_then(|table| {
                table
                    .0
                    .into_iter()
                    .map(|(index, entry)| {
                        T::try_from(entry)
                            .map(|value| (index, value))
                            .map_err(Error::msg)
                    })
                    .collect::<Result<HashMap<_, _>>>()
            })
            .map(Table)
    }
}
