//! Counting how often each line of a loaded file runs, which is what
//! `Coverage` reports.
//!
//! A file is measured only when it is loaded while measurement is on, the way
//! Ruby does it: the counting is set up as the file is read, so a file already
//! loaded is never counted.

use crate::ast::{Expression, Statement};
use crate::error::MetorexError;
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;
use indexmap::IndexMap;

/// One measurement run: the lines of each file it has seen, counted where a
/// line carries something to run and left out where it does not.
#[derive(Default)]
pub(crate) struct CoverageRun {
    /// Whether the run reports each mode under its own name, which is what
    /// naming any mode outright asks for.
    by_mode: bool,
    /// The modes the run reports, in the order Ruby lists them.
    modes: Vec<String>,
    /// Whether code handed to `eval` under a measured file's name is counted
    /// into that file.
    pub(crate) eval_too: bool,
    /// Each file measured, by the path it was loaded under, with one entry per
    /// line of the file.
    files: Vec<(String, Vec<Option<u64>>)>,
    /// Every method defined in a measured file while the run was on, with the
    /// span it was written across and how often it has been called.
    methods: Vec<MethodSpan>,
}

/// A method as the `methods` mode reports it: where it sits, and how often it
/// has run.
struct MethodSpan {
    file: String,
    owner: Object,
    name: String,
    start_line: usize,
    start_column: usize,
    end_line: usize,
    end_column: usize,
    calls: u64,
}

impl VirtualMachine {
    /// Start measuring, forgetting whatever an earlier run counted.
    pub(crate) fn coverage_start(&mut self, by_mode: bool, modes: Vec<String>, eval_too: bool) {
        self.coverage = Some(CoverageRun {
            by_mode,
            modes,
            eval_too,
            files: Vec::new(),
            methods: Vec::new(),
        });
    }

    pub(crate) fn coverage_stop(&mut self) {
        self.coverage = None;
    }

    /// Whether code handed to `eval` is counted into the file it names.
    pub(crate) fn coverage_counts_eval(&self) -> bool {
        self.coverage.as_ref().is_some_and(|run| run.eval_too)
    }

    /// Begin counting a file that is being loaded now. Every line carrying
    /// something to run starts at zero, and every other line is left out.
    pub(crate) fn coverage_note_file(&mut self, path: &str, source: &str, body: &[Statement]) {
        if self.coverage.is_none() {
            return;
        }
        let counted = source.lines().count().max(1);
        let mut lines = vec![None; counted];
        mark(body, &mut lines);
        if let Some(run) = self.coverage.as_mut() {
            run.files.retain(|(held, _)| held != path);
            run.files.push((path.to_string(), lines));
        }
    }

    /// Add the lines of code handed to `eval` under a file name, which `eval`
    /// coverage counts into that file. A name the run has not seen before
    /// becomes a file of its own, the way Ruby reports `eval("...", b, name)`.
    pub(crate) fn coverage_note_eval(&mut self, path: &str, body: &[Statement]) {
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if !run.eval_too {
            return;
        }
        let mut wanted = Vec::new();
        collect(body, &mut wanted);
        let Some(highest) = wanted.iter().max().copied() else {
            return;
        };
        if !run.files.iter().any(|(held, _)| held == path) {
            run.files.push((path.to_string(), Vec::new()));
        }
        let Some((_, lines)) = run.files.iter_mut().find(|(held, _)| held == path) else {
            return;
        };
        if lines.len() < highest {
            lines.resize(highest, None);
        }
        for line in wanted {
            if line >= 1 && lines[line - 1].is_none() {
                lines[line - 1] = Some(0);
            }
        }
    }

    /// Count one run of the line at `line` in the file being run now.
    pub(crate) fn coverage_count(&mut self, line: usize) {
        if line == 0 {
            return;
        }
        let Some(path) = self.current_source_file.clone() else {
            return;
        };
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        let Some((_, lines)) = run.files.iter_mut().find(|(held, _)| *held == path) else {
            return;
        };
        if let Some(Some(held)) = lines.get_mut(line - 1) {
            *held += 1;
        }
    }

    /// Record a method written in a measured file, which the `methods` mode
    /// reports whether or not it is ever called.
    pub(crate) fn coverage_note_method(
        &mut self,
        owner: Object,
        name: &str,
        start: crate::lexer::Position,
        end: crate::lexer::Position,
    ) {
        let Some(path) = self.current_source_file.clone() else {
            return;
        };
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if !run.files.iter().any(|(held, _)| *held == path) {
            return;
        }
        run.methods.push(MethodSpan {
            file: path,
            owner,
            name: name.to_string(),
            start_line: start.line,
            // Ruby counts columns from zero, and the span runs to just past
            // the `end` that closes the definition.
            start_column: start.column.saturating_sub(1),
            end_line: end.line,
            end_column: end.column.saturating_sub(1) + "end".len(),
            calls: 0,
        });
    }

    /// Count one call of a method the run is watching.
    pub(crate) fn coverage_count_method(&mut self, file: &str, name: &str, line: usize) {
        let Some(run) = self.coverage.as_mut() else {
            return;
        };
        if let Some(held) = run
            .methods
            .iter_mut()
            .find(|held| held.file == file && held.name == name && held.start_line == line)
        {
            held.calls += 1;
        }
    }

    /// What the run has counted so far, as the Hash `Coverage.result` hands
    /// back. `clear` sets every count back to zero without forgetting which
    /// lines are countable.
    pub(crate) fn coverage_report(&mut self, clear: bool) -> Result<Object, MetorexError> {
        let Some(run) = self.coverage.as_mut() else {
            return Ok(Object::dict(IndexMap::new()));
        };
        let by_mode = run.by_mode;
        let modes = run.modes.clone();
        let counts: Vec<(String, Vec<Option<u64>>)> = run.files.clone();
        let defined: Vec<(String, Object, String, [usize; 4], u64)> = run
            .methods
            .iter()
            .map(|held| {
                (
                    held.file.clone(),
                    held.owner.clone(),
                    held.name.clone(),
                    [
                        held.start_line,
                        held.start_column,
                        held.end_line,
                        held.end_column,
                    ],
                    held.calls,
                )
            })
            .collect();
        if clear {
            for (_, lines) in run.files.iter_mut() {
                for held in lines.iter_mut().filter(|held| held.is_some()) {
                    *held = Some(0);
                }
            }
            for held in run.methods.iter_mut() {
                held.calls = 0;
            }
        }

        let mut answer = IndexMap::new();
        for (path, lines) in &counts {
            let counted = Object::array(
                lines
                    .iter()
                    .map(|held| match held {
                        Some(times) => Object::Int(*times as i64),
                        None => Object::Nil,
                    })
                    .collect(),
            );
            let value = if by_mode {
                let mut held = IndexMap::new();
                for mode in &modes {
                    let named = format!(":{mode}");
                    match mode.as_str() {
                        "lines" => held.insert(named, counted.clone()),
                        "methods" => held.insert(named, self.methods_in(&defined, path)?),
                        _ => held.insert(named, Object::dict(IndexMap::new())),
                    };
                }
                Object::dict(held)
            } else {
                counted
            };
            answer.insert(path.clone(), value);
        }
        Ok(Object::dict(answer))
    }

    /// The methods written in one file, keyed the way the `methods` mode names
    /// them: owner, name, and the span the definition covers.
    fn methods_in(
        &mut self,
        defined: &[(String, Object, String, [usize; 4], u64)],
        path: &str,
    ) -> Result<Object, MetorexError> {
        let mut entries: IndexMap<String, Object> = IndexMap::new();
        let mut key_objects: IndexMap<String, Object> = IndexMap::new();
        for (_, owner, name, span, calls) in defined.iter().filter(|held| held.0 == path) {
            let key = Object::array(vec![
                owner.clone(),
                Object::symbol(name.clone()),
                Object::Int(span[0] as i64),
                Object::Int(span[1] as i64),
                Object::Int(span[2] as i64),
                Object::Int(span[3] as i64),
            ]);
            let slot = self.dict_slot_in(&entries, &key, false, Position::new(0, 0, 0))?;
            key_objects.insert(slot.clone(), key);
            entries.insert(slot, Object::Int(*calls as i64));
        }
        if !key_objects.is_empty() {
            // An Array key is not read back from its text, so the key object
            // itself travels beside the entry.
            entries.insert(
                "__MX_KEY_OBJECTS__".to_string(),
                Object::Dict(std::rc::Rc::new(std::cell::RefCell::new(key_objects))),
            );
        }
        Ok(Object::dict(entries))
    }
}

/// Mark every line of `body` that carries something to run, leaving a line
/// already marked at the count it has.
fn mark(body: &[Statement], lines: &mut [Option<u64>]) {
    let mut wanted = Vec::new();
    collect(body, &mut wanted);
    for line in wanted {
        if line >= 1 && line <= lines.len() && lines[line - 1].is_none() {
            lines[line - 1] = Some(0);
        }
    }
}

/// The lines of `body` that carry something to run.
fn collect(body: &[Statement], found: &mut Vec<usize>) {
    for statement in body {
        // `begin` is not a line of its own in Ruby's counting: what runs is
        // the body inside it.
        if !matches!(statement, Statement::Begin { .. } | Statement::Block { .. }) {
            found.push(statement.position().line);
        }
        for inner in nested_bodies(statement) {
            collect(inner, found);
        }
        for inner in nested_blocks(statement) {
            collect(inner, found);
        }
    }
}

/// The statement bodies a statement holds, which carry lines of their own.
fn nested_bodies(statement: &Statement) -> Vec<&[Statement]> {
    match statement {
        Statement::ClassDef { body, .. }
        | Statement::ModuleDef { body, .. }
        | Statement::MethodDef { body, .. }
        | Statement::FunctionDef { body, .. }
        | Statement::While { body, .. }
        | Statement::DoWhile { body, .. }
        | Statement::For { body, .. } => vec![body.as_slice()],
        Statement::Block { statements, .. } => vec![statements.as_slice()],
        Statement::If {
            then_branch,
            elsif_branches,
            else_branch,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![then_branch.as_slice()];
            for branch in elsif_branches {
                held.push(branch.body.as_slice());
            }
            if let Some(otherwise) = else_branch {
                held.push(otherwise.as_slice());
            }
            held
        }
        Statement::Unless {
            then_branch,
            else_branch,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![then_branch.as_slice()];
            if let Some(otherwise) = else_branch {
                held.push(otherwise.as_slice());
            }
            held
        }
        Statement::Match { cases, .. } | Statement::CaseIn { cases, .. } => {
            cases.iter().map(|case| case.body.as_slice()).collect()
        }
        Statement::Begin {
            body,
            rescue_clauses,
            else_clause,
            ensure_block,
            ..
        } => {
            let mut held: Vec<&[Statement]> = vec![body.as_slice()];
            for clause in rescue_clauses {
                held.push(clause.body.as_slice());
            }
            if let Some(otherwise) = else_clause {
                held.push(otherwise.as_slice());
            }
            if let Some(ensured) = ensure_block {
                held.push(ensured.as_slice());
            }
            held
        }
        _ => Vec::new(),
    }
}

/// The bodies of blocks written on a statement's own expressions, which run
/// line by line the same way.
fn nested_blocks(statement: &Statement) -> Vec<&[Statement]> {
    let mut found = Vec::new();
    for expression in own_expressions(statement) {
        gather_blocks(expression, &mut found);
    }
    found
}

fn own_expressions(statement: &Statement) -> Vec<&Expression> {
    match statement {
        Statement::Expression { expression, .. } | Statement::Match { expression, .. } => {
            vec![expression]
        }
        Statement::Assignment { target, value, .. } => vec![target, value],
        Statement::MultipleAssignment {
            targets, values, ..
        } => targets.iter().chain(values.iter()).collect(),
        Statement::Return {
            value: Some(held), ..
        }
        | Statement::Raise {
            exception: Some(held),
            ..
        } => vec![held],
        _ => Vec::new(),
    }
}

fn gather_blocks<'a>(expression: &'a Expression, found: &mut Vec<&'a [Statement]>) {
    match expression {
        Expression::Lambda { body, .. } | Expression::SingletonClass { body, .. } => {
            found.push(body.as_slice())
        }
        Expression::BeginRescue { body, .. } => found.push(body.as_slice()),
        Expression::MethodCall {
            receiver,
            arguments,
            trailing_block,
            ..
        } => {
            gather_blocks(receiver, found);
            for argument in arguments {
                gather_blocks(argument, found);
            }
            if let Some(block) = trailing_block {
                gather_blocks(block, found);
            }
        }
        _ => {}
    }
}
