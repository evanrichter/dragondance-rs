//! Export ranges of code coverage data to the [dragondance] Pin Helper file format
//!
//! ```
//! use dragondance::{Module, Trace};
//!
//! // Create a Trace with module info
//! let modules = [Module::new("abcd", 0x1000, 0x2000),
//!                Module::new("libc.so", 0x555000, 0x556000)];
//! let mut trace = Trace::new(&modules);
//!
//! // Add coverage events from your emulator, debugger, etc.
//! trace.add(0x1204, 3);
//! trace.add(0x1207, 12);
//!
//! // Write the coverage to a dragondance coverage file
//! trace.save("trace.dd").unwrap();
//! ```
//! [dragondance]: https://github.com/0ffffffffh/dragondance

use std::io::{Error, Write};

/// A collection of code coverage entries that can be [exported][`Trace::write`] in a
/// dragondance compatible format.
#[derive(Debug, Clone)]
pub struct Trace {
    modules: Vec<Module>,
    entries: Vec<Entry>,
}

/// A named executable object
#[derive(Debug, Clone, Copy)]
pub struct Module {
    name: &'static str,
    base: u64,
    end: u64,
}

impl Module {
    /// Create a new Module. Panics if end < base.
    pub fn new(name: &'static str, base: u64, end: u64) -> Self {
        assert!(base < end, "`base` must be before `end`");
        assert!(
            (end - base) <= u32::MAX as u64,
            "Module sizes > u32::MAX are not representable in dragondance format"
        );
        Self { name, base, end }
    }

    /// True if the given pc is within this module.
    pub fn contains(&self, pc: u64) -> bool {
        self.base <= pc && pc < self.end
    }
}

impl Trace {
    /// Create a new Trace by providing a slice of Modules.
    pub fn new(modules: &[Module]) -> Self {
        Self {
            modules: modules.to_vec(),
            entries: Vec::new(),
        }
    }

    /// Get the [`Module`] containing the given PC, or None.
    pub fn module_containing<'a>(&'a self, pc: u64) -> Option<&'a Module> {
        self.modules.iter().find(|m| m.contains(pc))
    }

    /// Add a coverage entry to the trace.
    ///
    /// * `pc`: The program counter executed
    /// * `size`: The length, in bytes, of the basic block. (Or instruction length if tracing
    ///           single instructions)
    pub fn add(&mut self, pc: u64, size: usize) {
        let size = size
            .try_into()
            .expect("Entry size is too large for DragonDance file format, must be <= u16::MAX");

        let entry = self
            .modules
            .iter()
            .enumerate()
            .find(|(_, m)| m.contains(pc))
            .map(|(id, module)| Entry {
                offset: (pc - module.base).try_into().unwrap(),
                size,
                module: id as u16 + 1,
            })
            .expect("No module found that contains PC");

        self.entries.push(entry);
    }

    /// Write the coverage trace in the [Dragondance Pintool Helper format].
    ///
    /// [Dragondance Pintool Helper format]: https://github.com/0ffffffffh/dragondance/issues/1#issuecomment-493699908
    pub fn write(&self, writer: &mut impl Write) -> Result<(), Error> {
        // write header
        writeln!(writer, "DDPH-PINTOOL")?;
        writeln!(
            writer,
            "EntryCount: {}, ModuleCount: {}",
            self.entries.len(),
            self.modules.len()
        )?;

        // write module table
        writeln!(writer, "MODULE_TABLE")?;
        for (number, Module { name, base, end }) in self.modules.iter().enumerate() {
            let number = number + 1;
            writeln!(writer, "{number}, {base:#x}, {end:#x}, {name}")?;
        }

        // entry table (binary trace data)
        writeln!(writer, "ENTRY_TABLE")?;
        for entry in &self.entries {
            entry.write(writer)?;
        }

        Ok(())
    }

    /// Save the coverage trace to a file.
    pub fn save(&self, path: impl AsRef<std::path::Path>) -> Result<(), Error> {
        let mut outfile = std::fs::File::create(path)?;
        self.write(&mut outfile)
    }
}

// A single coverage "event". Likely this will represent a single basic block of assembly that was
// executed.
#[derive(Debug, Clone, Copy)]
struct Entry {
    offset: u32,
    size: u16,
    module: u16,
}

impl Entry {
    fn write(&self, w: &mut impl Write) -> Result<(), Error> {
        let mut buf = [0; 12];

        buf[0..4].copy_from_slice(&self.offset.to_ne_bytes());
        buf[4..6].copy_from_slice(&self.size.to_ne_bytes());
        buf[6..8].copy_from_slice(&self.module.to_ne_bytes());

        // instruction count is not used by dragon dance at all, and it is unclear what it is
        let inst_count: u32 = 0;
        buf[8..12].copy_from_slice(&inst_count.to_ne_bytes());

        w.write_all(&buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let modules = [Module::new("abcd.so", 0x1000, 0x2000)];
        let mut trace = Trace::new(&modules);

        trace.add(0x1234, 1);

        let mut out = Vec::new();
        trace.write(&mut out).unwrap();

        assert!(!out.is_empty());
    }

    #[test]
    #[should_panic]
    fn add_out_of_bounds() {
        let modules = [Module::new("abcd.so", 0x1000, 0x2000)];
        let mut trace = Trace::new(&modules);
        trace.add(0xdead, 10);
    }
}
