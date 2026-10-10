// Loading a feature by name.

use super::*;

impl VirtualMachine {
    /// What `$LOAD_PATH.resolve_feature_path` answers: the kind of file a
    /// `require` of the name would load, `:rb` or `:so`, and its path, or nil
    /// when nothing would be found.
    pub(crate) fn resolve_feature_path(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // The prelude's `resolve_feature_path` hands over its one argument.
        let feature = arguments.first().cloned().unwrap_or(Object::Nil);
        let named = self.coerce_load_path(&feature, position)?;
        let named = self.expand_home_path(&named);
        let named_outright = std::path::Path::new(&named).is_absolute()
            || named.starts_with("./")
            || named.starts_with("../");
        let bases: Vec<std::path::PathBuf> = if named_outright {
            vec![std::path::PathBuf::new()]
        } else {
            self.load_path_directories()
                .into_iter()
                .map(|directory| {
                    let base = std::path::PathBuf::from(directory);
                    base.canonicalize().unwrap_or(base)
                })
                .collect()
        };
        let mut candidates = require_candidates(&named);
        if std::path::Path::new(&named).extension().is_none() {
            candidates.extend(
                NATIVE_EXTENSIONS
                    .iter()
                    .map(|extension| std::path::PathBuf::from(format!("{named}.{extension}"))),
            );
        }
        let answer = |kind: &str, path: String| {
            Object::array(vec![Object::symbol(kind.to_string()), Object::string(path)])
        };
        for base in &bases {
            for candidate in &candidates {
                let path = base.join(candidate);
                if path.is_file() {
                    let kind = if names_a_native_extension(&path) {
                        "so"
                    } else {
                        "rb"
                    };
                    let expanded = expanded_feature_path(&path);
                    return Ok(answer(kind, expanded.to_string_lossy().into_owned()));
                }
            }
        }
        // A library metorex carries is what a require of it loads when the
        // load path holds no file of that name.
        let plainly = named.strip_suffix(".rb").unwrap_or(&named);
        if BUILT_IN_FEATURES.contains(&plainly)
            || crate::vm::stdlib::embedded_library(plainly).is_some()
        {
            return Ok(answer("rb", format!("<metorex>/{plainly}.rb")));
        }
        Ok(Object::Nil)
    }

    pub(crate) fn require_feature(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // require(name) loads and executes a file from $LOAD_PATH
        if arguments.len() != 1 {
            return Err(MetorexError::runtime_error(
                format!("require() expects 1 argument, got {}", arguments.len()),
                crate::vm::utils::position_to_location(position),
            ));
        }

        let require_name = self.coerce_load_path(&arguments[0], position)?;
        // A leading `~` names the home directory, as the shell has it.
        let require_name = self.expand_home_path(&require_name);

        // Search $LOAD_PATH for the file
        let search_dirs = self.load_path_directories();

        // A library metorex provides itself is already loaded, so a
        // require of it answers false rather than looking for a file.
        let named_plainly = require_name
            .strip_suffix(".rb")
            .or_else(|| require_name.strip_suffix(".so"))
            .unwrap_or(&require_name);
        if BUILT_IN_FEATURES.contains(&named_plainly) {
            // What the interpreter carries is read the first time a
            // program names it, and it counts as having been there
            // all along, so the answer is that nothing was loaded.
            if let Some(source) = crate::vm::stdlib::embedded_library(named_plainly) {
                self.run_embedded_library(named_plainly, source)?;
            }
            return Ok(Object::Bool(false));
        }
        // An absolute path, or one written relative to the working
        // directory, names the file outright rather than being
        // searched for.
        let mut found_path = None;
        let direct = std::path::PathBuf::from(&require_name);
        // A path written from the working directory or from the root
        // names the file outright. Ruby does not look for it on the
        // load path as well, which is what keeps `./name` meaning the
        // one file it spells.
        let named_outright = direct.is_absolute()
            || require_name.starts_with("./")
            || require_name.starts_with("../");
        if named_outright {
            for candidate in require_candidates(&require_name) {
                if candidate.is_file() {
                    found_path = Some(candidate);
                    break;
                }
            }
        }
        // A name already answered from somewhere on the load path is
        // loaded, whatever the load path holds by now: the file it
        // was answered by is listed, and the directory it sits in is
        // still one the search would reach.
        if !named_outright {
            let listed = require_candidates(&require_name);
            for dir in &search_dirs {
                let base = std::path::PathBuf::from(dir);
                let base = base.canonicalize().unwrap_or(base);
                for candidate in &listed {
                    let named = base.join(candidate);
                    if self.feature_is_listed(&named.to_string_lossy()) {
                        return Ok(Object::Bool(false));
                    }
                }
            }
        }
        // Each ending is looked for along the whole load path before the
        // next, so a `.rb` file anywhere on it wins over a C extension in
        // an earlier directory, and either over a directory of the name.
        if !named_outright && found_path.is_none() {
            'search: for candidate in require_candidates(&require_name) {
                for dir in &search_dirs {
                    // A directory on the load path is named by the file it
                    // points at, while the name asked for is left as written.
                    let base = std::path::PathBuf::from(dir);
                    let base = base.canonicalize().unwrap_or(base);
                    let path = base.join(&candidate);
                    if path.is_file() {
                        found_path = Some(path);
                        break 'search;
                    }
                }
            }
        }
        // A file built for the machine rather than written in Ruby is
        // not something metorex can run. Ruby reports what the loader
        // said rather than the path it was looking for, so the error
        // names no path at all.
        let mut named_a_native_extension = found_path
            .as_ref()
            .is_some_and(|held| names_a_native_extension(held));
        // A path with no extension of its own may still name one of
        // those files, and naming it is refused the same way.
        if found_path.is_none() && !named_a_native_extension {
            named_a_native_extension = NATIVE_EXTENSIONS.iter().any(|held| {
                std::path::PathBuf::from(format!("{}.{}", require_name, held)).is_file()
            });
        }
        // A file another thread is part-way through loading is waited
        // for, so what it defines is there to read.
        if let Some(held) = &found_path
            && let Ok(canonical) = held.canonicalize()
        {
            self.wait_for_the_file_being_loaded(&canonical);
        }

        // A feature listed under the very name asked for has been
        // loaded, however that name reads as a path from here.
        if require_candidates(&require_name).iter().any(|candidate| {
            let named = candidate.to_string_lossy();
            named.ends_with(".rb") && self.feature_is_listed(&named)
        }) {
            return Ok(Object::Bool(false));
        }
        let resolved = match found_path {
            Some(p) => p,
            None => {
                // A name with no ending of its own counts too once
                // there is no file of that name to be found. A C extension
                // listed under the name does not stand in for the Ruby
                // library metorex carries under it, as `pathname.so` does not
                // for `pathname`.
                let carried = crate::vm::stdlib::embedded_library(&require_name).is_some();
                if require_candidates(&require_name).iter().any(|candidate| {
                    let named = candidate.to_string_lossy();
                    self.feature_is_listed(&named) && (!carried || named.ends_with(".rb"))
                }) {
                    return Ok(Object::Bool(false));
                }
                // A library metorex carries is used when the load path
                // holds no file of that name.
                if let Some(source) = crate::vm::stdlib::embedded_library(&require_name) {
                    let already = self.run_embedded_library(&require_name, source)?;
                    return Ok(Object::Bool(already));
                }
                if let Some(init) = crate::vm::capi::static_extension(&require_name) {
                    let first = self.run_static_extension(&require_name, init, position)?;
                    return Ok(Object::Bool(first));
                }
                // Raise a LoadError exception so Ruby-level rescue LoadError catches it.
                let exc = if named_a_native_extension {
                    crate::vm::errors::load_error_without_path(format!(
                        "cannot load such file -- {}",
                        require_name
                    ))
                } else {
                    crate::vm::errors::load_error(
                        format!("cannot load such file -- {}", require_name),
                        &require_name,
                    )
                };
                return Err(MetorexError::UncaughtException {
                    exception: exc.clone(),
                    location: crate::vm::utils::position_to_location(position),
                    message: format!("{}", exc),
                });
            }
        };

        let canonical_path = resolved.canonicalize().map_err(|e| {
            MetorexError::runtime_error(
                format!(
                    "Failed to canonicalize path '{}': {}",
                    resolved.display(),
                    e
                ),
                crate::vm::utils::position_to_location(position),
            )
        })?;

        // `$LOADED_FEATURES` is the source of truth: a spec that
        // restores it expects the next require to load the file again
        // and answer true.
        let canonical_str = canonical_path.to_string_lossy().into_owned();
        // An entry written from the working directory, or with `..`
        // left in it, names the same file as the path it expands to,
        // which is what decides whether the file is loaded already.
        let standing_for = expanded_feature_path(&resolved);
        let was_already_loaded = match self.globals().get("\"") {
            Some(Object::Array(features)) => features.borrow().iter().any(|feature| {
                let Object::String(name) = feature else {
                    return false;
                };
                *name.as_str() == *canonical_str
                    || expanded_feature_path(std::path::Path::new(&*name.as_str())) == standing_for
            }),
            _ => self.is_file_loaded(&canonical_path),
        };

        // A file the feature list already names is not run again,
        // whatever spelling it is listed under.
        if was_already_loaded {
            return Ok(Object::Bool(false));
        }
        if names_a_native_extension(&resolved) {
            self.mark_file_loaded(canonical_path.clone());
            let listed = standing_for.to_string_lossy().into_owned();
            let features = match self.globals().get("\"") {
                Some(Object::Array(features)) => Some(features),
                _ => None,
            };
            if let Some(features) = &features {
                features.borrow_mut().push(Object::string(listed.clone()));
            }
            if let Err(error) = self.load_native_extension(&canonical_path, position) {
                self.unmark_file_loaded(&canonical_path);
                if let Some(features) = &features {
                    features.borrow_mut().retain(
                        |feature| !matches!(feature, Object::String(held) if *held.as_str() == *listed),
                    );
                }
                return Err(error);
            }
            return Ok(Object::Bool(true));
        }
        self.load_call_site = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|file| file.display().to_string())
            })
            .map(|file| (file, position));
        self.execute_file(&resolved).map_err(|e| {
            crate::vm::errors::keep_exception(e, |message| {
                MetorexError::runtime_error(
                    format!("require('{}') — {}", require_name, message),
                    crate::vm::utils::position_to_location(position),
                )
            })
        })?;

        Ok(Object::Bool(!was_already_loaded))
    }

    pub(crate) fn require_relative_feature(
        &mut self,
        arguments: Vec<Object>,
        position: Position,
    ) -> Result<Object, MetorexError> {
        // require_relative(path) loads and executes a file relative to the current file
        if arguments.len() != 1 {
            return Err(MetorexError::runtime_error(
                format!(
                    "require_relative() expects 1 argument, got {}",
                    arguments.len()
                ),
                crate::vm::utils::position_to_location(position),
            ));
        }

        // A path may be named by anything that says how to read
        // itself as one, the way `require` takes it.
        let relative_path = self.coerce_load_path(&arguments[0], position)?;

        // Ruby resolves the path against the file the call was
        // written in, which is not the file being loaded when a
        // method defined elsewhere is what is running.
        let written_in = self
            .current_source_file
            .as_ref()
            .map(std::path::PathBuf::from)
            .filter(|path| path.is_file());
        let current_file = written_in
            .as_ref()
            .or_else(|| self.get_current_file())
            .ok_or_else(|| {
                MetorexError::runtime_error(
                    "require_relative cannot be used without a current file context (e.g., in REPL)"
                        .to_string(),
                    crate::vm::utils::position_to_location(position),
                )
            })?;

        // A file reached through a symlink counts as being written
        // where the link points, so what it asks for relative to
        // itself is found beside the real file.
        let current_file = current_file
            .canonicalize()
            .unwrap_or_else(|_| current_file.to_path_buf());
        let current_file = &current_file;
        // Resolve the relative path
        let resolved_path = crate::file_loader::resolve_relative_path(current_file, &relative_path)
            .map_err(|e| {
                MetorexError::runtime_error(
                    format!(
                        "require_relative('{}') — cannot resolve path: {}",
                        relative_path,
                        e.message()
                    ),
                    crate::vm::utils::position_to_location(position),
                )
            })?;
        // Ruby names the file by the path it expands to, with no
        // `.` or `..` left in it, which is what a LoadError reports
        // and what `$LOADED_FEATURES` lists.
        let resolved_path = crate::vm::loading::without_dot_components(&resolved_path);

        // Find the actual file path with extension auto-detection. A
        // missing file is a LoadError, the way Ruby reports one.
        let actual_path = match crate::file_loader::find_required_file_path(&resolved_path) {
            Ok(path) => path,
            Err(_) => {
                let feature = resolved_path.display().to_string();
                let mut message = format!("cannot load such file -- {}", feature);
                let suggestions = crate::file_loader::suggest_similar_files(&resolved_path);
                if !suggestions.is_empty() {
                    message.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                }
                return Err(MetorexError::UncaughtException {
                    exception: crate::vm::errors::load_error(message.clone(), &feature),
                    location: crate::vm::utils::position_to_location(position),
                    message,
                });
            }
        };

        // Canonicalize to get the absolute path for deduplication checking
        let canonical_path = actual_path.canonicalize().map_err(|e| {
            MetorexError::runtime_error(
                format!(
                    "Failed to canonicalize path '{}': {}",
                    actual_path.display(),
                    e
                ),
                crate::vm::utils::position_to_location(position),
            )
        })?;

        // `$LOADED_FEATURES` is the source of truth, the way it is
        // for `require`: a spec that restores it expects the next
        // call to load the file again and answer true.
        let listed = crate::vm::loading::without_dot_components(&actual_path)
            .display()
            .to_string();
        let was_already_loaded = match self.globals().get("\"") {
            Some(Object::Array(features)) => features.borrow().iter().any(
                |feature| matches!(feature, Object::String(name) if *name.as_str() == *listed),
            ),
            _ => self.is_file_loaded(&canonical_path),
        };

        // Execute the file (it will handle its own deduplication)
        self.load_call_site = self
            .current_source_file
            .clone()
            .or_else(|| {
                self.current_file
                    .as_ref()
                    .map(|file| file.display().to_string())
            })
            .map(|file| (file, position));
        self.execute_file(&actual_path).map_err(|e| {
            crate::vm::errors::keep_exception(e, |message| {
                MetorexError::runtime_error(
                    format!("require_relative('{}') — {}", relative_path, message),
                    crate::vm::utils::position_to_location(position),
                )
            })
        })?;

        // Return true if newly loaded, false if already loaded (Ruby behavior)
        Ok(Object::Bool(!was_already_loaded))
    }
}
