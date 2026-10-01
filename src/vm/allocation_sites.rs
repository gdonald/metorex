// Where each object was made while `ObjectSpace.trace_object_allocations`
// was on: the file, the line, and the method running there.

use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use crate::vm::core::WeakTarget;
use std::rc::Rc;

/// What the object space reports about where one object was made.
#[derive(Debug, Clone)]
pub(crate) struct AllocationSite {
    file: Option<String>,
    line: usize,
    class_path: Option<String>,
    method_id: Option<String>,
    generation: i64,
}

/// The address an object lives at, for the kinds a program allocates. The
/// immediate values and the symbols are never allocated, so they have none.
fn allocated_address(object: &Object) -> Option<usize> {
    match object {
        Object::Instance(held) => Some(Rc::as_ptr(held) as *const u8 as usize),
        Object::Array(held) => Some(Rc::as_ptr(held) as *const u8 as usize),
        Object::Dict(held) => Some(Rc::as_ptr(held) as *const u8 as usize),
        Object::String(held) => Some(Rc::as_ptr(held) as *const u8 as usize),
        _ => None,
    }
}

impl VirtualMachine {
    /// Note where `object` was made, when tracing is on.
    pub(crate) fn record_allocation(&mut self, object: &Object, position: Position) {
        if self.allocation_tracing_depth == 0 || position.prelude {
            return;
        }
        let Some(address) = allocated_address(object) else {
            return;
        };
        let file = match &self.current_source_file {
            Some(written) => Some(
                self.reported_spelling(&std::path::PathBuf::from(written))
                    .display()
                    .to_string(),
            ),
            None => self
                .reported_current_file()
                .map(|path| path.display().to_string()),
        };
        let (class_path, method_id) = match self.enclosing_method_owner() {
            Some((owner, defined)) => (owner, Some(defined)),
            None => (None, None),
        };
        let site = AllocationSite {
            file,
            line: position.line,
            class_path,
            method_id,
            generation: self.gc_collection_count(),
        };
        self.allocation_sites
            .insert(address, (WeakTarget::of(object), site));
    }

    /// Where `object` was made, when it was made while tracing was on and
    /// the records have not been cleared since.
    pub(crate) fn allocation_site_of(&self, object: &Object) -> Option<&AllocationSite> {
        let address = allocated_address(object)?;
        let (target, site) = self.allocation_sites.get(&address)?;
        // An address a freed object left behind may now hold an object made
        // while tracing was off, which has no place recorded.
        let alive = target.reach()?;
        (allocated_address(&alive) == Some(address)).then_some(site)
    }

    /// `ObjectSpace.__allocation_tracing__(command)`: `:start` and `:stop`
    /// nest by count, `:clear` forgets every record, and `:site` answers
    /// what was recorded for the second argument.
    pub(crate) fn allocation_tracing_command(&mut self, arguments: &[Object]) -> Object {
        let command = match arguments.first() {
            Some(Object::Symbol(name)) => name.as_str().to_string(),
            _ => String::new(),
        };
        match command.as_str() {
            "start" => {
                self.allocation_tracing_depth += 1;
                Object::Nil
            }
            "stop" => {
                self.allocation_tracing_depth = self.allocation_tracing_depth.saturating_sub(1);
                Object::Nil
            }
            "clear" => {
                self.allocation_sites.clear();
                Object::Nil
            }
            _ => match arguments
                .get(1)
                .and_then(|held| self.allocation_site_of(held))
            {
                Some(site) => Object::array(vec![
                    site.file.clone().map_or(Object::Nil, Object::string),
                    Object::Int(site.line as i64),
                    site.class_path.clone().map_or(Object::Nil, Object::string),
                    site.method_id.clone().map_or(Object::Nil, Object::symbol),
                    Object::Int(site.generation),
                ]),
                None => Object::Nil,
            },
        }
    }
}
