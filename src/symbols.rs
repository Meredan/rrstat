use crate::maps;
use addr2line::Context;
use anyhow::{anyhow, Context as _, Result};
use gimli::{EndianReader, RunTimeEndian};
use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

// Type alias for the complex Context type from addr2line
type Addr2LineContext = Context<EndianReader<RunTimeEndian, Rc<[u8]>>>;

#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub function: Option<String>,
    pub file: Option<String>,
    pub line: Option<u32>,
}

pub struct SymbolResolver {
    contexts: HashMap<u32, Addr2LineContext>,
    cache: HashMap<(u32, u64), SymbolInfo>,
}

impl SymbolResolver {
    pub fn new() -> Self {
        Self {
            contexts: HashMap::new(),
            cache: HashMap::new(),
        }
    }

    fn load_binary(&self, pid: u32) -> Result<Addr2LineContext> {
        // profiling only the main binary of the PID.
        let exe_path = std::fs::read_link(format!("/proc/{}/exe", pid))
            .with_context(|| format!("Failed to read /proc/{}/exe", pid))?;

        let file = fs::File::open(&exe_path)
            .with_context(|| format!("Failed to open binary {:?}", exe_path))?;
        
        let data = unsafe { memmap2::Mmap::map(&file)? };
        let object = object::File::parse(&*data)?;

        let context = Context::new(&object)?;
        Ok(context)
    }

    fn check_cache(&self, pid: u32, addr: u64) -> Option<SymbolInfo> {
        if let Some(info) = self.cache.get(&(pid, addr)).cloned() {
            return Some(info);
        }
        None
    }

    fn update_cache(&mut self, pid: u32, addr: u64, info: SymbolInfo) {
        if self.cache.len() > 50_000 {
            self.cache.clear();
        }
        self.cache.insert((pid, addr), info);
    }

    fn get_context(&mut self, pid: u32) -> Result<&Addr2LineContext> {
        if !self.contexts.contains_key(&pid) {
            let context = self.load_binary(pid)?;
            self.contexts.insert(pid, context);
        }
        self.contexts.get(&pid).ok_or_else(|| anyhow!("Failed to get context after insertion"))
    }


    pub fn resolve(&mut self, pid: u32, addr: u64) -> Result<SymbolInfo> {
        if let Some(info) = self.check_cache(pid, addr) {
            return Ok(info);
        }
        
        // Find mapping to calculate relative address
        let mapping = maps::find_mapping_for_address(pid, addr)?
            .ok_or_else(|| anyhow!("No executable mapping found for address 0x{:x}", addr))?;
        
        let relative_addr = addr - mapping.start + mapping.offset;

        let info = {
            let context = self.get_context(pid)?;
            
            // addr2line 0.21.0 find_frames returns LookupResult
            let mut frames = match context.find_frames(relative_addr) {
                addr2line::LookupResult::Output(result) => result?,
                _ => return Err(anyhow!("Deferred loading not supported")),
            };

            if let Some(frame) = frames.next()? {
                let function = frame.function.as_ref().and_then(|f| {
                    f.demangle().ok().map(|d| d.to_string())
                });
                
                let (file, line) = if let Some(loc) = frame.location {
                    (loc.file.map(|f| f.to_string()), loc.line)
                } else {
                    (None, None)
                };
                
                Some(SymbolInfo {
                    function,
                    file,
                    line,
                })
            } else {
                None
            }
        };

        if let Some(info) = info {
            self.update_cache(pid, addr, info.clone());
            Ok(info)
        } else {
            Err(anyhow!("No frame found for address 0x{:x}", addr))
        }
    }
}
