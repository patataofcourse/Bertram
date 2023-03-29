use std::{fs::File, io, path::Path};

use csv::{DeserializeRecordsIter, Reader, Trim};
use serde::Deserialize;
use serde_hex::{SerHex, Strict};

use crate::crash::{saltwater::Region, CrashInfo, ModdingEngine};

#[derive(Debug, Clone)]
pub struct CrashAnalysis {
    pub oob_pc: bool,
    pub ctype: ModdingEngine,
    pub pc: Function,
    pub lr: Function,
    pub call_stack: Vec<Function>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub reg_pos: u32,
    pub func_pos: u32,
    pub symbol: String,
}

#[derive(Debug, Deserialize)]
pub struct CsvSymbol {
    #[serde(alias = "Name")]
    pub name: String,
    #[serde(alias = "Location", with = "SerHex::<Strict>")]
    pub location: u32,
    #[serde(alias = "Namespace")]
    pub namespace: Option<String>,
}

impl CsvSymbol {
    pub fn full_name(&self) -> String {
        match self.namespace.as_deref() {
            Some("Global") | None => self.name.clone(),
            Some(c) => c.to_owned() + "::" + &self.name,
        }
    }
}

type SymbolIter<'a> = DeserializeRecordsIter<'a, File, CsvSymbol>;

struct Symbols {
    megamix_reader: Reader<File>,
    saltwater_reader: Option<Reader<File>>,
}

impl Symbols {
    pub fn from_paths(
        megamix_path: impl AsRef<Path>,
        saltwater_path: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let mut builder = csv::ReaderBuilder::new();
        builder.trim(Trim::Fields);

        Ok(Self {
            megamix_reader: builder.from_path(megamix_path)?,
            saltwater_reader: match builder.from_path(saltwater_path){
                Ok(c) => Some(c),
                Err(e) => if let csv::ErrorKind::Io(e_io) = e.kind() && let io::ErrorKind::NotFound = e_io.kind() {
                    None
                } else {
                    Err(e)?
                },
            },
        })
    }

    pub fn megamix(&mut self) -> SymbolIter {
        self.megamix_reader.deserialize()
    }

    pub fn saltwater(&mut self) -> Option<SymbolIter> {
        Some(self.saltwater_reader.as_mut()?.deserialize())
    }
}

impl CrashAnalysis {
    const DISPLAY_PC_IF_OOB: bool = false;

    pub fn from(crash: &CrashInfo) -> io::Result<Self> {
        let symbols = match &crash.engine {
            ModdingEngine::RHMPatch => Symbols::from_paths("sym/rhm.us.csv", ""),
            ModdingEngine::SpiceRack(_, version, region) => Symbols::from_paths(
                format!(
                    "sym/rhm.{}.csv",
                    match region {
                        Region::JP => "jp",
                        Region::US => "us",
                        Region::EU => "eu",
                        Region::KR => "kr",
                        Region::UNK => Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Cannot analyze unknown region"
                        ))?,
                    }
                ),
                format!(""),
            ),
        };
        todo!();
    }
}
