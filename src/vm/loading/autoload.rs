// The constants registered against a file that defines them.

use super::*;

impl VirtualMachine {
    /// Return the autoload path registered for `name` on `class_rc` (walking
    /// ancestors), unless that file appears in `$LOADED_FEATURES` ($") —
    /// in which case the autoload registration is silently dropped and
    /// `None` is returned. Per MRI: `autoload?` and `defined?` treat an
    /// autoload as cleared once its target file has been required directly.
    /// `$LOADED_FEATURES` (rather than the internal `loaded_files` set) is
    /// the source of truth here so spec helpers that snapshot/restore `$"`
    /// across tests really do reset autoload visibility. When the direct
    /// require finished without actually defining the constant, the name
    /// is also moved into the "unrealized autoload" registry so it stays
    /// in `Module#constants` (per MRI's behavior for the direct-require
    /// path, which differs from the const-access trigger path).
    pub(crate) fn effective_autoload(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> Option<String> {
        let path = class_rc.lookup_autoload(name)?;
        // If this autoload is currently loading, behavior depends on
        // which thread is asking. The loading thread itself observes
        // the autoload as cleared (Ruby treats the entry as already
        // satisfied from inside its own load). Other threads still see
        // the registered path — so a concurrent `autoload?` returns the
        // path while the loading thread's `autoload?` returns nil.
        let current = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        for (cls, n, loader) in &self.autoload_loading {
            if Rc::ptr_eq(cls, class_rc) && n == name {
                let same_thread = match (loader, &current) {
                    (Object::Nil, Object::Nil) => true,
                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                };
                if same_thread {
                    return None;
                } else {
                    return Some(path);
                }
            }
        }
        if self.path_in_loaded_features(&path) {
            class_rc.remove_autoload(name);
            // Only the direct-require path keeps the cleared name in
            // `Module#constants` (via the unrealized list). When clearing
            // happens inside a const-access autoload trigger, the trigger
            // itself is responsible for the post-load bookkeeping.
            if self.autoload_const_access_depth == 0 && class_rc.get_class_var(name).is_none() {
                class_rc.mark_unrealized_autoload(name);
            }
            return None;
        }
        Some(path)
    }

    /// Read-only variant of `effective_autoload` for constant-presence
    /// checks (`const_defined?` / `const_get` lookup): whether `class_rc`
    /// itself has a pending autoload registration for `name`. Unlike
    /// `effective_autoload` it never clears the registration, checks only
    /// the receiver (callers walk ancestors themselves), and treats a
    /// same-thread in-progress load or an already-loaded file as not
    /// pending.
    pub(crate) fn autoload_pending(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> bool {
        let Some(path) = class_rc.get_autoload(name) else {
            return false;
        };
        let current = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        for (cls, n, loader) in &self.autoload_loading {
            if Rc::ptr_eq(cls, class_rc) && n == name {
                let same_thread = match (loader, &current) {
                    (Object::Nil, Object::Nil) => true,
                    (Object::Instance(a), Object::Instance(b)) => Rc::ptr_eq(a, b),
                    _ => false,
                };
                return !same_thread;
            }
        }
        !self.path_in_loaded_features(&path)
    }

    /// Whether `path` (after canonicalization) is currently listed in
    /// `$LOADED_FEATURES`. Used by `effective_autoload` and by the
    /// `autoload?` natives.
    pub(crate) fn path_in_loaded_features(&self, path: &str) -> bool {
        let abs = std::path::Path::new(path);
        let canonical = match abs.canonicalize() {
            Ok(p) => p.to_string_lossy().into_owned(),
            Err(_) => return false,
        };
        if let Some(Object::Array(arr)) = self.globals().get("\"") {
            arr.borrow()
                .iter()
                .any(|o| matches!(o, Object::String(s) if *s.as_str() == *canonical))
        } else {
            false
        }
    }

    /// The object a wrapped load runs against: an Object of its own that
    /// answers the way the top-level main does, carrying the module the load
    /// was wrapped in ahead of whatever main itself was extended with.
    pub(crate) fn a_main_of_its_own(
        &mut self,
        wrapper: &Rc<crate::class::Class>,
    ) -> Option<Object> {
        let Some(Object::Instance(main)) = self.globals().get("__main__") else {
            return None;
        };
        let object_class = Rc::clone(&main.borrow().class);
        let stand_in = Object::Instance(crate::object::Instance::new(object_class));
        // The copy is a main of the program's own, which is what decides
        // whether `using` written on it is allowed.
        if let Object::Instance(held) = &stand_in {
            held.borrow_mut()
                .set_var(MAIN_STAND_IN.to_string(), Object::Bool(true));
        }
        let singleton = self.singleton_class_of(&stand_in);
        // What main says of itself, which the copy says too, and whatever
        // main was extended with, which the copy carries after the wrapper.
        let main_singleton = main.borrow().singleton_class.borrow().clone();
        if let Some(held) = main_singleton {
            for name in held.method_names() {
                if let Some(method) = held.find_own_method(&name) {
                    // The copy owns what it answers with, so `method(:to_s)`
                    // names the copy's own singleton class rather than the
                    // one it was taken from.
                    let mut carried = (*method).clone();
                    carried.owner = None;
                    carried.owner_class = None;
                    singleton.define_method(name, Rc::new(carried));
                }
            }
            for module in held.mixin_chain().into_iter().rev() {
                singleton.add_mixin(module);
            }
        }
        singleton.add_mixin(Rc::clone(wrapper));
        Some(stand_in)
    }

    /// Whether `named` is listed in `$LOADED_FEATURES` as it stands.
    pub(crate) fn feature_is_listed(&self, named: &str) -> bool {
        match self.globals().get("\"") {
            Some(Object::Array(features)) => features
                .borrow()
                .iter()
                .any(|held| matches!(held, Object::String(text) if *text.as_str() == *named)),
            _ => false,
        }
    }

    /// Hand control over while another thread is part-way through loading
    /// `path`. A load that never finishes, because the thread running it was
    /// killed, gives up after a while rather than holding this thread.
    pub(crate) fn wait_for_the_file_being_loaded(&mut self, path: &std::path::Path) {
        let named = path.to_string_lossy().into_owned();
        let here = self.running_thread();
        let waited_enough = std::time::Instant::now() + AUTOLOAD_WAIT_CEILING;
        while self
            .loading_paths
            .iter()
            .any(|(held, thread)| held == &named && !on_the_same_thread(thread, &here))
        {
            if std::time::Instant::now() >= waited_enough {
                return;
            }
            self.wait_for_other_threads(crate::lexer::Position::new(0, 0, 0));
        }
    }

    /// Wait while another thread is part-way through the autoload of `name`,
    /// so the name reads as what the load defines rather than as the half
    /// built module the load has reached so far.
    pub(crate) fn settle_pending_autoload(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) {
        if self.autoload_loading.is_empty() {
            return;
        }
        let here = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        let loading_elsewhere = self.autoload_loading.iter().any(|(cls, n, thread)| {
            Rc::ptr_eq(cls, class_rc) && n == name && !on_the_same_thread(thread, &here)
        });
        if loading_elsewhere {
            self.wait_for_the_autoload(class_rc, name);
        }
    }

    /// Hand control over until the thread loading `name` has finished, so
    /// what the load defines is there to read. A load that never finishes,
    /// because the thread running it was killed, gives up after a while
    /// rather than holding this thread forever.
    pub(crate) fn wait_for_the_autoload(&mut self, class_rc: &Rc<crate::class::Class>, name: &str) {
        let waited_enough = std::time::Instant::now() + AUTOLOAD_WAIT_CEILING;
        while self
            .autoload_loading
            .iter()
            .any(|(cls, n, _)| Rc::ptr_eq(cls, class_rc) && n == name)
        {
            if std::time::Instant::now() >= waited_enough {
                return;
            }
            self.wait_for_other_threads(crate::lexer::Position::new(0, 0, 0));
        }
    }

    /// What an autoload left behind for `name`, which for a name written at
    /// the top level of the loaded file is a global rather than a class
    /// variable of Object.
    pub(crate) fn autoloaded_value(
        &self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> Option<Object> {
        class_rc.get_class_var(name).or_else(|| {
            (class_rc.name() == "Object")
                .then(|| self.globals().get(name))
                .flatten()
        })
    }

    /// If `class_rc` (or an ancestor) has an autoload registration for
    /// `name`, fire it: load the file, drop the registration on success, and
    /// return the now-defined constant. On load failure, restore the
    /// registration and unmark the file so a retry can re-execute. Returns
    /// `Ok(None)` when there is no autoload and the caller should fall
    /// through to a NameError or other fallback.
    pub(crate) fn try_autoload_constant(
        &mut self,
        class_rc: &Rc<crate::class::Class>,
        name: &str,
    ) -> Result<Option<Object>, MetorexError> {
        let Some(path) = class_rc.lookup_autoload(name) else {
            return Ok(None);
        };
        // If this autoload is currently loading on this thread, just
        // return whatever class_var the load has deposited so far —
        // keeping the entry alive for other threads' visibility. The
        // `try_autoload_constant` invocation that started the load is
        // responsible for cleaning up after the load completes.
        let loading_on = self
            .autoload_loading
            .iter()
            .find(|(cls, n, _)| Rc::ptr_eq(cls, class_rc) && n == name)
            .map(|(_, _, thread)| thread.clone());
        if let Some(loader) = loading_on {
            let here = self
                .thread_current_stack
                .last()
                .cloned()
                .unwrap_or(Object::Nil);
            // A thread that asks for a name another thread is part-way
            // through loading waits for that load to finish, so it reads a
            // module that is built rather than one that is half built.
            if !on_the_same_thread(&loader, &here) {
                self.wait_for_the_autoload(class_rc, name);
                return Ok(self.autoloaded_value(class_rc, name));
            }
            return Ok(class_rc.get_class_var(name));
        }
        // If the registered file has already been loaded *and* it
        // deposited a value for this constant, the autoload is
        // satisfied: drop the entry and return the stored value.
        //
        // If the constant is still missing, two cases apply:
        //  - The file is *currently* mid-execution (a class body inside
        //    that file is reopening this scope and asking for the
        //    constant before the assignment line has run). Skip
        //    re-loading — the load that's already running will get
        //    there. Returning None lets the caller fall through to a
        //    NameError, which the in-flight body either ignores or
        //    handles (typical Ruby autoload-during-require pattern).
        //  - The file finished and didn't define this constant. That
        //    means several autoloads point at the same path and the
        //    finished load only defined sibling names. Re-load so the
        //    body re-runs and defines this one too. Defeat
        //    `execute_file`'s dedup by clearing the path from `$"` and
        //    from the internal `loaded_files` set.
        let mut reloading = false;
        if self.path_in_loaded_features(&path) {
            if let Some(val) = class_rc.get_class_var(name) {
                class_rc.remove_autoload(name);
                return Ok(Some(val));
            }
            // Top-level `module Foo` / `class Foo` lands on globals
            // rather than on Object's class_vars in our model. When
            // the autoload was registered on Object, peek there too
            // before deciding to re-load.
            if class_rc.name() == "Object"
                && let Some(val) = self.globals().get(name)
            {
                class_rc.remove_autoload(name);
                return Ok(Some(val));
            }
            let canonical_str = std::path::Path::new(&path)
                .canonicalize()
                .ok()
                .map(|p| p.to_string_lossy().into_owned());
            let in_progress = canonical_str
                .as_ref()
                .is_some_and(|c| self.loading_paths.iter().any(|(held, _)| held == c))
                || self.loading_paths.iter().any(|(held, _)| held == &path);
            if in_progress {
                return Ok(None);
            }
            if let Some(canonical) = canonical_str {
                self.unmark_file_loaded(&PathBuf::from(canonical));
            }
            if let Some(Object::Array(arr)) = self.globals().get("\"") {
                arr.borrow_mut()
                    .retain(|o| !matches!(o, Object::String(s) if *s.as_str() == *path));
            }
            reloading = true;
        }
        // Don't remove the registration up-front. MRI keeps the constant
        // visible in `Module#constants` for the duration of the load; if
        // the load body asks `defined?` or `autoload?` about the same
        // name, `effective_autoload` (consulted by those methods) will
        // see the file in `$LOADED_FEATURES` and clear the entry on its
        // own.
        let p = std::path::Path::new(&path);
        // Autoload load runs at top level (like `require`): a `module Foo`
        // at the file's top should reopen ::Foo, not nest inside the
        // currently-executing class/module body. Save and clear the
        // def-scope stack for the duration of the load so lexical-nesting
        // logic in execute_class_def / execute_module_def sees an empty
        // outer scope.
        let saved_def_scope = std::mem::take(&mut self.def_scope_stack);
        self.autoload_const_access_depth += 1;
        if reloading {
            self.autoload_reload_depth += 1;
        }
        let loader_thread = self
            .thread_current_stack
            .last()
            .cloned()
            .unwrap_or(Object::Nil);
        self.autoload_loading
            .push((Rc::clone(class_rc), name.to_string(), loader_thread));
        // MRI dispatches autoload's load through `main.require(path)` so
        // singleton-method mocks on `main` (mspec's `main.should_receive
        // (:require)`) catch the call. Try that path first; fall back to
        // the internal loader when `main` doesn't have a `require` method
        // (e.g. when the singleton method has been removed or never set
        // up via Kernel inclusion).
        let main_require = self
            .globals()
            .get("TOPLEVEL_BINDING")
            .and_then(|tb| match tb {
                Object::Binding(b) => b.receiver.clone(),
                _ => None,
            })
            .and_then(|main| {
                self.lookup_method(&main, "require")
                    .map(|(cls, m)| (main, cls, m))
            });
        let load_result: Result<(), MetorexError> = if let Some((main, cls, method)) = main_require
        {
            self.invoke_method(
                cls,
                method,
                main,
                vec![Object::string(path.clone())],
                crate::lexer::Position::new(0, 0, 0),
            )
            .map(|_| ())
        } else if names_a_c_extension(p) {
            self.require_feature(
                vec![Object::string(path.clone())],
                crate::lexer::Position::new(0, 0, 0),
            )
            .map(|_| ())
        } else if p.is_absolute() {
            self.execute_file(p).map(|_| ())
        } else {
            self.require_library(&path)
        };
        self.autoload_const_access_depth -= 1;
        if reloading {
            self.autoload_reload_depth -= 1;
        }
        self.autoload_loading
            .retain(|(cls, n, _)| !(Rc::ptr_eq(cls, class_rc) && n == name));
        self.def_scope_stack = saved_def_scope;
        if let Err(err) = load_result {
            class_rc.set_autoload(name, &path);
            if let Ok(canonical) = p.canonicalize() {
                self.unmark_file_loaded(&canonical);
            }
            // Translate "file missing" runtime errors into a Ruby-level
            // LoadError so `rescue LoadError` works the way MRI's autoload
            // does. Autoload's other errors (RuntimeError, NameError, etc.)
            // pass through unchanged.
            let msg = err.message().to_string();
            if msg.contains("File not found") || msg.contains("cannot load such file") {
                let exc = crate::vm::errors::load_error(msg.clone(), &path);
                return Err(MetorexError::UncaughtException {
                    exception: exc,
                    location: SourceLocation::new(0, 0, 0),
                    message: msg,
                });
            }
            return Err(err);
        }
        // Const-access path: load completed. Drop the registration now —
        // either the constant was defined (a real class_var assignment
        // already cleared the autoload via set_class_var, but a no-op
        // remove is harmless) or it wasn't (MRI fully drops the name from
        // `#constants` for the const-access trigger, distinct from the
        // direct-require path which retains an unrealized marker).
        class_rc.remove_autoload(name);
        let mut value = class_rc.get_class_var(name);
        // Top-level `module Foo` / `class Foo` lands on globals rather
        // than on Object's class_vars in our model. When the autoload
        // was registered on Object, fall back to globals so the lookup
        // surfaces the freshly-defined constant.
        if value.is_none() && class_rc.name() == "Object" {
            value = self.globals().get(name);
        }
        // MRI's verbose-mode warning when an autoload-triggered file
        // completed but didn't define the named constant. Only emitted
        // under `$VERBOSE = true`; routed through `$stderr` so the
        // mspec `complain` matcher (which swaps `$stderr` for an
        // `IOStub`) captures it.
        if value.is_none() && matches!(self.globals().get("VERBOSE"), Some(Object::Bool(true))) {
            let mod_name = if class_rc.name().is_empty() {
                "main".to_string()
            } else {
                class_rc.ruby_name()
            };
            let msg = format!(
                "Expected {} to define {}::{} but it didn't",
                path, mod_name, name,
            );
            self.emit_warning_to_stderr(&msg, crate::lexer::Position::new(0, 0, 0));
        }
        Ok(value)
    }
}

/// The instance variable that marks the main a wrapped load runs against, so
/// it is taken for main the way the program's own is.
pub(crate) const MAIN_STAND_IN: &str = "__main_stand_in__";

/// How long a thread waits for another thread's autoload before carrying on
/// without it, so a load whose thread was killed does not hold it forever.
pub(crate) const AUTOLOAD_WAIT_CEILING: std::time::Duration = std::time::Duration::from_secs(20);

/// Whether two values name the same Thread.
pub(crate) fn on_the_same_thread(one: &Object, other: &Object) -> bool {
    match (one, other) {
        (Object::Instance(held), Object::Instance(here)) => Rc::ptr_eq(held, here),
        // Neither names a thread of its own, so both are the thread the
        // program started on.
        (Object::Nil, Object::Nil) => true,
        _ => false,
    }
}

/// Whether an autoload path names a C extension, written with its ending or
/// standing beside one built for this platform, which `require` loads.
fn names_a_c_extension(path: &std::path::Path) -> bool {
    if crate::vm::native_functions::names_a_native_extension(path) {
        return true;
    }
    let built = format!(
        "{}.{}",
        path.display(),
        crate::vm::native_functions::PLATFORM_EXTENSION
    );
    path.extension().is_none()
        && !path.with_extension("rb").is_file()
        && std::path::Path::new(&built).is_file()
}
