use std::{
    ffi::CString,
    fmt::Display,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use anyhow::anyhow;
use bytestream::{ByteOrder::LittleEndian as LE, StreamReader};
use csv::{DeserializeRecordsIter, Position, Reader, Trim, Writer};
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
    pub ctype: ModdingEngine,
    pub pc: MaybeFunction,
    pub lr: MaybeFunction,
    pub call_stack: Vec<MaybeFunction>,
}

impl CrashAnalysis {
    pub fn region(&self) -> Region {
        self.ctype.region()
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub reg_pos: u32,
    pub func_pos: u32,
    pub symbol: String,
}

#[derive(Debug, Clone)]
pub enum MaybeFunction {
    Function(Function),
    Oob(u32),
}

impl MaybeFunction {
    pub fn get_raw_pos(&self) -> u32 {
        match self {
            MaybeFunction::Function(c) => c.reg_pos,
            MaybeFunction::Oob(pos) => *pos,
        }
    }
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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

pub struct Symbols {
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

pub fn get_megamix_bounds() -> anyhow::Result<Vec<CsvBounds>> {
    let mut builder = csv::ReaderBuilder::new();
    builder.trim(Trim::Fields);

    let mut megamix_bounds = builder.from_path("sym/bounds.csv")?;
    Ok(megamix_bounds
        .deserialize::<CsvBounds>()
        .try_collect::<Vec<_>>()?)
}

impl Symbols {
    pub fn from_paths(
        megamix_path: impl AsRef<Path>,
        saltwater_path: impl AsRef<Path>,
    ) -> anyhow::Result<Self> {
        let mut builder = csv::ReaderBuilder::new();
        builder.trim(Trim::Fields);
        builder.has_headers(true);

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

    pub fn megamix(&mut self) -> anyhow::Result<SymbolIter> {
        self.megamix_reader.reset()?;
        Ok(self.megamix_reader.deserialize())
    }

    pub fn saltwater(&mut self) -> anyhow::Result<Option<SymbolIter>> {
        self.saltwater_reader
            .as_mut()
            .map(|c| {
                c.reset()?;
                Ok(Some(c.deserialize()))
            })
            .unwrap_or(Ok(None))
    }

    pub fn init_bounds(&mut self, region: Region) -> anyhow::Result<()> {
        let Some(a) = get_megamix_bounds()?
            .iter()
            .find(|c| region.matches(&c.version))
            .cloned()
        else {
            Err(anyhow!("Bounds file doesn't include {:?} region", region))?
        };
        self.megamix_end = Some(a.rodata);

        self.saltwater_end = if let Some(mut sw_syms) = self.saltwater()? {
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
        let Some(megamix_end) = self.megamix_end else {
            Err(anyhow!("Tried to get a symbol with uninitialized bounds!"))?
        };

        if pos >= 0x00100000 && pos < megamix_end {
            let mm_syms = self.megamix()?;
            let mut current_sym: Option<(u32, String)> = None;
            for sym in mm_syms {
                let sym = sym?;
                if let Some(c) = &current_sym
                    && sym.location < c.0
                {
                    unreachable!("This should never happen!")
                }
                if sym.location > pos {
                    break;
                }
                current_sym = Some((sym.location, sym.full_name()))
            }
            Ok(current_sym.map(|c| Function {
                reg_pos: pos,
                func_pos: c.0,
                symbol: c.1,
            }))
        } else if pos >= 0x07000000
            && let Some(sw_syms) = self.saltwater()?
            && pos <= self.saltwater_end.unwrap()
        {
            let mut current_sym: Option<(u32, String)> = None;
            for sym in sw_syms {
                let sym = sym?;
                if let Some(c) = &current_sym
                    && sym.location < c.0
                {
                    unreachable!("This should never happen!")
                }
                if sym.location > pos {
                    break;
                }
                current_sym = Some((sym.location, sym.full_name()))
            }
            Ok(current_sym.map(|c| Function {
                reg_pos: pos,
                func_pos: c.0,
                symbol: c.1,
            }))
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

            // ctrplugin symbols are sorted, and we don't need data stuff, so just yeet that
            if name == "_TEXT_END" {
                break;
            }
        }

        Ok(())
    }
}

impl CrashAnalysis {
    const DISPLAY_PC_IF_OOB: bool = false;
    const DISPLAY_LR_IF_OOB: bool = false;
    const DISPLAY_CALL_STACK_IF_OOB: bool = true;

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

        let pc = if let Some(c) = symbols.find_symbol(crash.pc)? {
            MaybeFunction::Function(c)
        } else {
            MaybeFunction::Oob(crash.pc)
        };
        let lr = if let Some(c) = symbols.find_symbol(crash.lr)? {
            MaybeFunction::Function(c)
        } else {
            MaybeFunction::Oob(crash.lr)
        };
        let call_stack = crash
            .call_stack
            .as_ref()
            .unwrap_or(&vec![])
            .iter()
            .map(|pos| {
                symbols.find_symbol(*pos).map(|c| {
                    if let Some(c) = c {
                        MaybeFunction::Function(c)
                    } else {
                        MaybeFunction::Oob(*pos)
                    }
                })
            })
            .try_collect()?;
        Ok(Self {
            pc,
            lr,
            call_stack,
            ctype: crash.engine.clone(),
        })
    }
}

impl Display for CrashAnalysis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
                "Crash analysis for {}:\n",
                "@ {:08x} -> {:08x} (@ PC -> LR)\n\n",
                "Call stack:\n",
                "{}",
                "{}",
                "{}",
            ),
            match &self.ctype {
                ModdingEngine::RHMPatch => "RHMPatch".to_string(),
                ModdingEngine::SpiceRack(_, ver, region) => format!("Saltwater {ver} ({region})"),
            },
            self.pc.get_raw_pos(),
            self.lr.get_raw_pos(),
            if let MaybeFunction::Function(c) = &self.pc {
                format!(
                    "  PC ({:08x}): {} ({:08x})\n",
                    c.reg_pos, c.symbol, c.func_pos
                )
            } else if Self::DISPLAY_PC_IF_OOB {
                format!("  PC ({:08x}): out of bounds!\n", self.pc.get_raw_pos())
            } else {
                String::new()
            },
            if let MaybeFunction::Function(c) = &self.lr {
                format!(
                    "  LR ({:08x}): {} ({:08x})\n",
                    c.reg_pos, c.symbol, c.func_pos
                )
            } else if Self::DISPLAY_LR_IF_OOB {
                format!("  LR ({:08x}): out of bounds!\n", self.lr.get_raw_pos())
            } else {
                String::new()
            },
            {
                let mut out = String::new();
                for (i, elmt) in self.call_stack.iter().enumerate() {
                    if let MaybeFunction::Function(c) = elmt {
                        out += &format!(
                            "  Call stack {} ({:08x}): {} ({:08x})\n",
                            i + 1,
                            c.reg_pos,
                            c.symbol,
                            c.func_pos
                        );
                    } else if Self::DISPLAY_CALL_STACK_IF_OOB {
                        out += &format!(
                            "  Call stack {} ({:08x}): out of bounds!\n",
                            i + 1,
                            elmt.get_raw_pos()
                        )
                    }
                }
                out
            }
        )
    }
}

#[cfg(feature = "bot")]
use serenity::builder::CreateEmbed;

#[cfg(feature = "bot")]
impl CrashAnalysis {
    pub fn as_serenity_embed(&self, embed: CreateEmbed) -> CreateEmbed {
        embed
            .title(format!(
                "Crash analysis for {}:",
                match &self.ctype {
                    ModdingEngine::RHMPatch => "RHMPatch".to_string(),
                    ModdingEngine::SpiceRack(_, ver, region) =>
                        format!("Saltwater {ver} ({region})"),
                }
            ))
            .description(format!(
                "@ {:08x} -> {:08x} (@ PC -> LR)\n\n",
                self.pc.get_raw_pos(),
                self.lr.get_raw_pos()
            ))
            .field(
                "Call stack",
                format!(
                    "{}{}{}",
                    if let MaybeFunction::Function(c) = &self.pc {
                        format!(
                            "PC ({:08x}): {} ({:08x})\n",
                            c.reg_pos, c.symbol, c.func_pos
                        )
                    } else if Self::DISPLAY_PC_IF_OOB {
                        format!("PC ({:08x}): out of bounds!\n", self.pc.get_raw_pos())
                    } else {
                        String::new()
                    },
                    if let MaybeFunction::Function(c) = &self.lr {
                        format!(
                            "LR ({:08x}): {} ({:08x})\n",
                            c.reg_pos, c.symbol, c.func_pos
                        )
                    } else if Self::DISPLAY_LR_IF_OOB {
                        format!("LR ({:08x}): out of bounds!\n", self.lr.get_raw_pos())
                    } else {
                        String::new()
                    },
                    {
                        let mut out = String::new();
                        for (i, elmt) in self.call_stack.iter().enumerate() {
                            if let MaybeFunction::Function(c) = elmt {
                                out += &format!(
                                    "Call stack {} ({:08x}): {} ({:08x})\n",
                                    i + 1,
                                    c.reg_pos,
                                    c.symbol,
                                    c.func_pos
                                );
                            } else if Self::DISPLAY_CALL_STACK_IF_OOB {
                                out += &format!(
                                    "Call stack {} ({:08x}): out of bounds!\n",
                                    i + 1,
                                    elmt.get_raw_pos()
                                )
                            }
                        }
                        out
                    }
                ),
                false,
            )
    }
}

trait SeekToStart: Sized {
    fn reset(&mut self) -> anyhow::Result<()>;
}

impl SeekToStart for Reader<File> {
    fn reset(&mut self) -> anyhow::Result<()> {
        let mut position = Position::new();
        position.set_byte(0);
        self.seek(position)?;
        self.read_record(&mut csv::StringRecord::new())?;
        Ok(())
    }
}
