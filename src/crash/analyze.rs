use std::{fs::File, io, path::Path};

use csv::{DeserializeRecordsIter, Reader, Trim};
use serde::Deserialize;
use serde_hex::{SerHex, Strict};

use crate::crash::{saltwater::{Region, SWDVersion}, CrashInfo, ModdingEngine};

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
            saltwater_reader: if saltwater_path.as_ref().as_os_str().is_empty() {None} else {Some(builder.from_path(saltwater_path)?)},
        })
    }

    pub fn megamix(&mut self) -> SymbolIter {
        self.megamix_reader.deserialize()
    }

    pub fn saltwater(&mut self) -> Option<SymbolIter> {
        Some(self.saltwater_reader.as_mut()?.deserialize())
    }

    pub fn find_symbol(&mut self, pos: u32, has_saltwater: bool) -> Option<Function> {
        // 1. find bounds (bounds.csv for megamix, location "Global::_text_end" in the saltwater symbols)
        // 2:
        //   if it's in megamix bounds, look through megamix symbols
        //   if it's in saltwater bounds, look through saltwater symbols if given
        //   otherwise, return None
        
        todo!();
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
                format!("sym/sw.{}.csv", match version {
                    SWDVersion::Debug{commit_hash} => commit_hash.clone(),
                    SWDVersion::Release{major, minor, patch} => format!("{major}.{minor}{}", if *patch != 0 {format!(".{patch}")} else {"".to_string()})
                }),
            ),
        };
        todo!();
    }
}
