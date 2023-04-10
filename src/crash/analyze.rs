use std::{
    ffi::CString,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::anyhow;
use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};
use csv::{DeserializeRecordsIter, Reader, Trim, Writer};
use grep_matcher::{Captures, Matcher};
use grep_regex::RegexMatcher;
use serde::{Deserialize, Serialize};
use serde_hex::{SerHex, Strict};

use crate::crash::{
    saltwater::{Region, SWDVersion},
    CrashInfo, ModdingEngine,
};

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

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct CsvBounds {
    #[serde(alias = "Version")]
    pub version: String,
    #[serde(alias = "Code offset", with = "SerHex::<Strict>")]
    pub code: u32,
    #[serde(alias = "Rodata offset", with = "SerHex::<Strict>")]
    pub rodata: u32,
    #[serde(alias = "Data offset", with = "SerHex::<Strict>")]
    pub data: u32,
    #[serde(alias = "BSS start", alias = "BSS offset", with = "SerHex::<Strict>")]
    pub bss_offset: u32,
    #[serde(alias = "BSS size", with = "SerHex::<Strict>")]
    pub bss_size: u32,
}

type SymbolIter<'a> = DeserializeRecordsIter<'a, File, CsvSymbol>;

struct Symbols {
    megamix_reader: Reader<File>,
    saltwater_reader: Option<Reader<File>>,
    megamix_end: Option<u32>,
    saltwater_end: Option<u32>,
}

pub fn get_3gx_commit_hash(f: &mut (impl Read + Seek)) -> anyhow::Result<Option<String>> {
    let ref_finder = RegexMatcher::new(r"rev ([0-9a-f]{7})")?;
    let mut captures = ref_finder.new_captures()?;
    let mut buf = vec![];
    f.read_to_end(&mut buf)?;
    ref_finder.captures(&buf, &mut captures)?;
    f.seek(SeekFrom::Start(0))?;

    if let Some(a) = captures.get(1) {
        Ok(Some(
            String::from_utf8(buf[a.start()..a.end()].to_owned()).unwrap(),
        ))
    } else {
        Ok(None)
    }
}

impl Symbols {
    pub fn from_paths(
        megamix_path: impl AsRef<Path>,
        saltwater_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let mut builder = csv::ReaderBuilder::new();
        builder.trim(Trim::Fields);

        Ok(Self {
            megamix_reader: builder.from_path(megamix_path)?,
            saltwater_reader: if saltwater_path.as_ref().as_os_str().is_empty() {
                None
            } else {
                Some(builder.from_path(saltwater_path)?)
            },
            megamix_end: None,
            saltwater_end: None,
        })
    }

    pub fn megamix(&mut self) -> SymbolIter {
        self.megamix_reader.deserialize()
    }

    pub fn saltwater(&mut self) -> Option<SymbolIter> {
        Some(self.saltwater_reader.as_mut()?.deserialize())
    }

    pub fn init_bounds(&mut self, region: Region) -> anyhow::Result<()> {
        let mut builder = csv::ReaderBuilder::new();
        builder.trim(Trim::Fields);

        let mut megamix_bounds = builder.from_path("sym/bounds.csv")?;
        let Some(Ok(a)) = megamix_bounds.deserialize::<CsvBounds>().find(|c| {
            let Ok(bound) = c else { return false };
            region.matches(&bound.version)
        }) else { Err(anyhow!("Bounds file doesn't include {:?} region", region))? };
        self.megamix_end = Some(a.rodata);

        self.saltwater_end = if let Some(mut sw_syms) = self.saltwater() {
                if let Some(Ok(a)) = sw_syms.find(|c| {
                    if let Ok(c) = c {
                        c.full_name() == "_TEXT_END"
                    } else {
                        false
                    }
                }) {
                    Some(a.location)
                } else {
                    Err(anyhow!(
                        "Saltwater symbols file doesn't contain _TEXT_END symbol"
                    ))?
                }
        } else {
            None
        };
        Ok(())
    }

    pub fn find_symbol(&mut self, pos: u32) -> anyhow::Result<Option<Function>> {
        let Some(megamix_end) = self.megamix_end else {Err(anyhow!("Tried to get a symbol with uninitialized bounds!"))?};

        if pos >= 0x00100000 && pos < megamix_end {
            // TODO: if it's in megamix bounds, look through megamix symbols
            let mut mm_syms = self.megamix();
            todo!();
        } else if let Some(mut sw_syms) = self.saltwater() && pos >= 0x07000000 && pos <= self.saltwater_end.unwrap() {
            // TODO: if it's in saltwater bounds, look through saltwater symbols if given
            todo!();
        } else {
            Ok(None)
        }
    }

    pub fn ctrplugin_symbols_to_csv<F: Read + Seek, W: Write>(
        plg: &mut F,
        csv: &mut W,
        demangle: bool,
    ) -> anyhow::Result<()> {
        let mut magic = [0u8; 8];
        plg.read_exact(&mut magic)?;
        if &magic != b"3GX$0002" {
            Err(anyhow!("not a compatible .3gx file",))?
        }
        plg.seek(SeekFrom::Start(0x88))?;

        let num_symbols = u32::read_from(plg, LE)? as u64;
        let symbols_offset = u32::read_from(plg, LE)? as u64;
        let name_table = u32::read_from(plg, LE)? as u64;

        let mut writer = Writer::from_writer(csv);

        for i in 0..num_symbols {
            plg.seek(SeekFrom::Start(symbols_offset + 0xC * i))?;

            let location = u32::read_from(plg, LE)?;
            plg.seek(SeekFrom::Current(4))?; // size, flags
            let name_pos = u32::read_from(plg, LE)? as u64;

            plg.seek(SeekFrom::Start(name_table + name_pos))?;
            let mut name = vec![];
            loop {
                let c = u8::read_from(plg, LE)?;
                if c == 0 {
                    break;
                }
                name.push(c);
            }
            let Ok(name) = CString::new(name)?.into_string() else {
                Err(anyhow!("could not read symbol name"))?
            };

            if demangle {
                //TODO: demangle symbol names
            }

            writer.serialize(CsvSymbol {
                name: name.clone(),
                location,
                namespace: None,
            })?;

            if name == "_TEXT_END" {
                break;
            }
        }

        Ok(())
    }
}

impl CrashAnalysis {
    const DISPLAY_PC_IF_OOB: bool = false;

    pub fn from(crash: &CrashInfo) -> anyhow::Result<Self> {
        let region;
        let mut symbols = match &crash.engine {
            ModdingEngine::RHMPatch => {
                region = Region::US;
                Symbols::from_paths("sym/rhm.us.csv", "")?
            }
            ModdingEngine::SpiceRack(_, version, region_) => {
                region = *region_;
                Symbols::from_paths(
                    format!(
                        "sym/rhm.{}.csv",
                        match region {
                            Region::JP => "jp",
                            Region::US => "us",
                            Region::EU => "eu",
                            Region::KR => "kr",
                            Region::UNK => Err(anyhow!("Cannot analyze unknown region"))?,
                        }
                    ),
                    format!(
                        "sym/sw.{}.csv",
                        match version {
                            SWDVersion::Debug { commit_hash } => "_".to_string() + commit_hash,
                            SWDVersion::Release {
                                major,
                                minor,
                                patch,
                            } => format!(
                                "{major}.{minor}{}",
                                if *patch != 0 {
                                    format!(".{patch}")
                                } else {
                                    "".to_string()
                                }
                            ),
                        }
                    ),
                )?
            }
        };
        symbols.init_bounds(region)?;
        symbols.find_symbol(crash.pc)?;
        todo!();
    }
}
