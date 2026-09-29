// The frames a backtrace reads, and the names they carry.

use super::*;

impl VirtualMachine {
    /// The VM call stack as Location objects, outermost call last, the way
    /// `caller_locations(0)` reports them.
    pub(crate) fn caller_location_objects(&mut self, position: Position) -> Vec<Object> {
        use crate::object::Instance;
        use std::rc::Rc;
        let loc_class = self.backtrace_location_class();
        let current_file = self
            .reported_current_file()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let stack = self.call_stack();
        let mut locations = Vec::with_capacity(stack.len() + 1);
        // The call stack records where each frame was entered from, so the
        // line `caller_locations` itself sits on is not among them. Ruby
        // counts it as the innermost location, which is what level 0 names.
        let here = Instance::new(Rc::clone(&loc_class));
        here.borrow_mut()
            .set_var("lineno".to_string(), Object::Int(position.line as i64));
        here.borrow_mut()
            .set_var("path".to_string(), Object::string(current_file.clone()));
        let here_absolute = match self.absolute_path_for(&current_file) {
            Some(resolved) => Object::string(resolved),
            None => Object::Nil,
        };
        here.borrow_mut()
            .set_var("absolute_path".to_string(), here_absolute);
        let frames: Vec<_> = stack.iter().rev().collect();
        // Where each frame was called from, with a call made inside the core
        // library standing for the place that reached it: Ruby names the
        // program's own file rather than `<internal:...>`.
        let called_from: Vec<(String, i64)> = frame_call_sites(&frames, &current_file);
        // A frame's own name labels the location it is running at, and Ruby
        // names a block by the scope holding it: `block in <main>`.
        let label_at = |index: usize| -> String { frame_label_at(&frames, index) };
        here.borrow_mut()
            .set_var("label".to_string(), Object::string(label_at(0)));
        locations.push(Object::Instance(here));
        for (index, frame) in frames.iter().enumerate() {
            // A frame with no recorded call site was never called from
            // anywhere — the file body itself — so it is not a caller.
            if frame.location().is_none() {
                continue;
            }
            let (path, line) = called_from[index].clone();
            // A frame entered from nowhere in particular records no line,
            // and there is no call for a backtrace to name there.
            if line == 0 {
                continue;
            }
            let inst = Instance::new(Rc::clone(&loc_class));
            inst.borrow_mut()
                .set_var("lineno".to_string(), Object::Int(line));
            let absolute = match self.absolute_path_for(&path) {
                Some(resolved) => Object::string(resolved),
                None => Object::Nil,
            };
            inst.borrow_mut()
                .set_var("path".to_string(), Object::string(path));
            inst.borrow_mut()
                .set_var("absolute_path".to_string(), absolute);
            // A frame records where it was called from, so its location pairs
            // with the name of the frame below it: the one that made the call.
            // A frame records where it was called from, so its location
            // pairs with the name of the frame below it: the one that made
            // the call.
            inst.borrow_mut()
                .set_var("label".to_string(), Object::string(label_at(index + 1)));
            locations.push(Object::Instance(inst));
        }
        locations
    }

    /// The caller locations a `caller`/`caller_locations` argument list names.
    /// `(start, length)` drops `start` frames and keeps `length` of them; a
    /// Range says which frames to keep directly. Dropping more than there are
    /// answers None, which both report as nil.
    pub(crate) fn sliced_caller_locations(
        &mut self,
        arguments: &[Object],
        position: Position,
    ) -> Result<Option<Vec<Object>>, MetorexError> {
        let all = self.caller_location_objects(position);
        let Some((skip, length)) = self.caller_slice_bounds(arguments, all.len(), 1) else {
            return Ok(None);
        };
        let mut kept: Vec<Object> = all.into_iter().skip(skip).collect();
        if let Some(length) = length {
            kept.truncate(length);
        }
        Ok(Some(kept))
    }

    /// Where a slice of a backtrace starts and how long it is, from the
    /// arguments naming it. None when it starts past the end, which Ruby
    /// answers as nil rather than an empty list.
    pub(crate) fn caller_slice_bounds(
        &mut self,
        arguments: &[Object],
        total: usize,
        default_skip: usize,
    ) -> Option<(usize, Option<usize>)> {
        let (skip, length) = match arguments.first() {
            Some(Object::Range { .. }) if arguments.len() == 1 => {
                self.range_bounds_for(&arguments[0], total)?
            }
            Some(Object::Int(number)) => (
                (*number).max(0) as usize,
                match arguments.get(1) {
                    Some(Object::Int(limit)) => Some((*limit).max(0) as usize),
                    _ => None,
                },
            ),
            _ => (default_skip, None),
        };
        if skip > total {
            return None;
        }
        Some((skip, length))
    }

    /// The (skip, length) a Range argument names over `total` frames, or None
    /// when it starts past the end.
    fn range_bounds_for(&mut self, value: &Object, total: usize) -> Option<(usize, Option<usize>)> {
        let Object::Range {
            start,
            end,
            exclusive,
            ..
        } = value
        else {
            return None;
        };
        let resolve = |bound: &Object, default: i64| -> i64 {
            match bound {
                Object::Int(number) => *number,
                _ => default,
            }
        };
        let first = resolve(start, 0);
        let first = if first < 0 {
            (total as i64 + first).max(0)
        } else {
            first
        } as usize;
        if first > total {
            return None;
        }
        let last = match end.as_ref() {
            Object::Nil => total as i64 - 1,
            bound => {
                let last = resolve(bound, total as i64 - 1);
                let last = if last < 0 { total as i64 + last } else { last };
                if *exclusive { last - 1 } else { last }
            }
        };
        let length = (last - first as i64 + 1).max(0) as usize;
        Some((first, Some(length)))
    }
}

/// The name the frame at `index` reads in a backtrace. A block is named for
/// the scope it was written in and for how many blocks deep it sits there.
pub(crate) fn frame_label_at(frames: &[&crate::vm::CallFrame], index: usize) -> String {
    let Some(frame) = frames.get(index) else {
        return "<main>".to_string();
    };
    let name = frame.name().to_string();
    let depth = if name == "<block>" {
        frame.block_depth().max(1)
    } else {
        frame.block_depth()
    };
    let held = if name == "<block>" {
        frame.written_in().unwrap_or("<main>").to_string()
    } else {
        name
    };
    let held = backtrace_label(&held);
    match depth {
        0 => held,
        1 => format!("block in {held}"),
        counted => format!("block ({counted} levels) in {held}"),
    }
}

/// The name a backtrace entry reads. A method defined on one object alone is
/// named by itself: the singleton class holding it has no name a reader would
/// know, so only the method's own name is written.
pub(crate) fn backtrace_label(name: &str) -> String {
    // The body of a file that was required is named for being that, since
    // the file it belongs to is written alongside the label anyway.
    if name.starts_with("<file:") {
        return "<top (required)>".to_string();
    }
    if name.starts_with("#<Class:#<") {
        return match name.rfind('#') {
            Some(at) if at + 1 < name.len() => name[at + 1..].to_string(),
            _ => name.to_string(),
        };
    }
    // A method written in `class << Name` belongs to Name, and that is the
    // name a reader knows it by.
    if let Some(rest) = name.strip_prefix("#<Class:")
        && let Some(at) = rest.find(">.")
    {
        return format!("{}{}", &rest[..at], &rest[at + 1..]);
    }
    name.to_string()
}

/// Where each frame was called from, innermost first. A frame reached from
/// the core library's own Ruby source stands for the place that reached it,
/// so a backtrace names the program's file rather than `<internal:...>`.
pub(crate) fn frame_call_sites(
    frames: &[&crate::vm::CallFrame],
    current_file: &str,
) -> Vec<(String, i64)> {
    let mut sites: Vec<(String, i64)> = frames
        .iter()
        .map(|frame| match frame.location() {
            // Frame locations are "line:column" or "file:line:column".
            Some(written) => {
                let parts: Vec<&str> = written.rsplitn(3, ':').collect();
                let line = parts
                    .get(1)
                    .and_then(|held| held.parse::<i64>().ok())
                    .unwrap_or(0);
                let path = match parts.get(2) {
                    Some(path) if !path.is_empty() => (*path).to_string(),
                    // The frame records the file its call site sits in, which
                    // is where the location belongs.
                    _ => frame
                        .source_file()
                        .map(|file| file.to_string())
                        .unwrap_or_else(|| current_file.to_string()),
                };
                (path, line)
            }
            None => (
                frame
                    .source_file()
                    .map(|file| file.to_string())
                    .unwrap_or_else(|| current_file.to_string()),
                0,
            ),
        })
        .collect();
    // Walking outward, an internal call site takes the one below it, which is
    // the nearest place in the program itself.
    let mut carried: Option<(String, i64)> = None;
    for site in sites.iter_mut().rev() {
        if site.0.starts_with(crate::vm::INTERNAL_FILE_PREFIX) {
            if let Some(held) = &carried {
                *site = held.clone();
            }
        } else {
            carried = Some(site.clone());
        }
    }
    sites
}
