// Prism's parser, which build.rs compiles in from vendor/prism, as the native
// functions prism's Ruby library is handed its serialized results by.

use super::*;
use std::os::raw::{c_char, c_int, c_void};

type FgetsCallback = unsafe extern "C" fn(*mut c_char, c_int, *mut c_void) -> *mut c_char;
type FeofCallback = unsafe extern "C" fn(*mut c_void) -> c_int;

unsafe extern "C" {
    fn pm_version() -> *const c_char;
    fn pm_buffer_sizeof() -> usize;
    fn pm_buffer_init(buffer: *mut c_void) -> bool;
    fn pm_buffer_value(buffer: *const c_void) -> *const c_char;
    fn pm_buffer_length(buffer: *const c_void) -> usize;
    fn pm_buffer_free(buffer: *mut c_void);
    fn pm_serialize_parse(buffer: *mut c_void, source: *const u8, size: usize, data: *const c_char);
    fn pm_serialize_lex(buffer: *mut c_void, source: *const u8, size: usize, data: *const c_char);
    fn pm_serialize_parse_lex(
        buffer: *mut c_void,
        source: *const u8,
        size: usize,
        data: *const c_char,
    );
    fn pm_serialize_parse_comments(
        buffer: *mut c_void,
        source: *const u8,
        size: usize,
        data: *const c_char,
    );
    fn pm_serialize_parse_stream(
        buffer: *mut c_void,
        stream: *mut c_void,
        stream_fgets: FgetsCallback,
        stream_feof: FeofCallback,
        data: *const c_char,
    );
    fn pm_parse_success_p(source: *const u8, size: usize, data: *const c_char) -> bool;
    fn pm_string_query_local(source: *const u8, length: usize, encoding: *const c_char) -> c_int;
    fn pm_string_query_constant(source: *const u8, length: usize, encoding: *const c_char)
    -> c_int;
    fn pm_string_query_method_name(
        source: *const u8,
        length: usize,
        encoding: *const c_char,
    ) -> c_int;
}

/// A `pm_buffer_t`, held as the bytes Prism says it takes, and freed when it
/// is dropped.
struct PrismBuffer {
    storage: Vec<u64>,
}

impl PrismBuffer {
    fn new() -> Self {
        let words = unsafe { pm_buffer_sizeof() }.div_ceil(std::mem::size_of::<u64>());
        let mut made = PrismBuffer {
            storage: vec![0; words.max(1)],
        };
        unsafe { pm_buffer_init(made.pointer()) };
        made
    }

    fn pointer(&mut self) -> *mut c_void {
        self.storage.as_mut_ptr().cast()
    }

    fn bytes(&mut self) -> Vec<u8> {
        let buffer = self.pointer();
        unsafe {
            let length = pm_buffer_length(buffer);
            if length == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(pm_buffer_value(buffer).cast::<u8>(), length).to_vec()
        }
    }
}

impl Drop for PrismBuffer {
    fn drop(&mut self) {
        unsafe { pm_buffer_free(self.pointer()) };
    }
}

/// What a stream being parsed is read through: the VM that calls `gets` and
/// `eof?` on it, the text read so far, and an error raised while reading.
struct StreamReading<'vm> {
    vm: &'vm mut VirtualMachine,
    stream: Object,
    position: Position,
    source: Vec<u8>,
    error: Option<MetorexError>,
}

/// `fgets` for Prism: the next line of the stream, at most `size - 1` bytes,
/// written into `string` with a NUL after it.
unsafe extern "C" fn read_stream_line(
    string: *mut c_char,
    size: c_int,
    stream: *mut c_void,
) -> *mut c_char {
    let reading = unsafe { &mut *stream.cast::<StreamReading>() };
    if reading.error.is_some() || size <= 1 {
        return std::ptr::null_mut();
    }
    let asked = Object::Int(i64::from(size - 1));
    match reading.vm.send_to_object(
        reading.stream.clone(),
        "gets",
        vec![asked],
        reading.position,
    ) {
        Ok(Object::String(line)) => {
            let bytes = crate::vm::native_methods::string_methods::binary_bytes(&line);
            let length = bytes.len().min(size as usize - 1);
            unsafe {
                std::ptr::copy_nonoverlapping(bytes.as_ptr(), string.cast::<u8>(), length);
                *string.add(length) = 0;
            }
            reading.source.extend_from_slice(&bytes[..length]);
            string
        }
        Ok(_) => std::ptr::null_mut(),
        Err(error) => {
            reading.error = Some(error);
            std::ptr::null_mut()
        }
    }
}

/// `feof` for Prism: whether the stream has nothing left.
unsafe extern "C" fn stream_at_end(stream: *mut c_void) -> c_int {
    let reading = unsafe { &mut *stream.cast::<StreamReading>() };
    if reading.error.is_some() {
        return 1;
    }
    match reading
        .vm
        .send_to_object(reading.stream.clone(), "eof?", Vec::new(), reading.position)
    {
        Ok(answer) => c_int::from(answer.is_truthy()),
        Err(error) => {
            reading.error = Some(error);
            1
        }
    }
}

/// The options Prism reads, or none when no options were packed.
fn options_pointer(options: &[u8]) -> *const c_char {
    if options.is_empty() {
        std::ptr::null()
    } else {
        options.as_ptr().cast()
    }
}

/// The bytes of a String argument.
fn string_bytes(argument: Option<&Object>, position: Position) -> Result<Vec<u8>, MetorexError> {
    match argument {
        Some(Object::String(text)) => Ok(crate::vm::native_methods::string_methods::binary_bytes(
            text,
        )),
        other => {
            let named = other.map_or("nil".to_string(), |held| held.type_name().to_string());
            Err(crate::vm::errors::simple_exception(
                "TypeError",
                &format!("wrong argument type {named} (expected String)"),
                position,
            ))
        }
    }
}

impl VirtualMachine {
    /// The native functions prism's Ruby library calls, by name.
    pub(crate) fn call_prism_function(
        &mut self,
        name: &str,
        arguments: &[Object],
        position: Position,
    ) -> Result<Object, MetorexError> {
        match name {
            "__prism_version__" => {
                let version = unsafe { std::ffi::CStr::from_ptr(pm_version()) };
                Ok(Object::string(version.to_string_lossy().into_owned()))
            }
            // `__prism_serialize__(kind, source, options)` answers what
            // Prism's serializer writes for one of its four readings.
            "__prism_serialize__" => {
                let kind = match arguments.first() {
                    Some(Object::Symbol(kind)) => kind.as_str().to_string(),
                    _ => String::new(),
                };
                let serialize = match kind.as_str() {
                    "parse" => pm_serialize_parse,
                    "lex" => pm_serialize_lex,
                    "parse_lex" => pm_serialize_parse_lex,
                    "parse_comments" => pm_serialize_parse_comments,
                    _ => {
                        return Err(crate::vm::errors::simple_exception(
                            "ArgumentError",
                            &format!("unknown prism serialization: {kind}"),
                            position,
                        ));
                    }
                };
                let source = string_bytes(arguments.get(1), position)?;
                let options = string_bytes(arguments.get(2), position)?;
                let mut buffer = PrismBuffer::new();
                unsafe {
                    serialize(
                        buffer.pointer(),
                        source.as_ptr(),
                        source.len(),
                        options_pointer(&options),
                    )
                };
                Ok(crate::vm::native_methods::pack_format::bytes_to_string(
                    &buffer.bytes(),
                ))
            }
            // `__prism_serialize_stream__(stream, options)` reads the stream
            // a line at a time, as far as Prism asks, and answers the text it
            // read and the serialized parse of it.
            "__prism_serialize_stream__" => {
                let stream = arguments.first().cloned().unwrap_or(Object::Nil);
                let options = string_bytes(arguments.get(1), position)?;
                let mut buffer = PrismBuffer::new();
                let mut reading = StreamReading {
                    vm: self,
                    stream,
                    position,
                    source: Vec::new(),
                    error: None,
                };
                unsafe {
                    pm_serialize_parse_stream(
                        buffer.pointer(),
                        (&mut reading as *mut StreamReading).cast(),
                        read_stream_line,
                        stream_at_end,
                        options_pointer(&options),
                    )
                };
                if let Some(error) = reading.error {
                    return Err(error);
                }
                Ok(Object::array(vec![
                    crate::vm::native_methods::pack_format::bytes_to_string(&reading.source),
                    crate::vm::native_methods::pack_format::bytes_to_string(&buffer.bytes()),
                ]))
            }
            "__prism_parse_success__" => {
                let source = string_bytes(arguments.first(), position)?;
                let options = string_bytes(arguments.get(1), position)?;
                let succeeded = unsafe {
                    pm_parse_success_p(source.as_ptr(), source.len(), options_pointer(&options))
                };
                Ok(Object::Bool(succeeded))
            }
            // `__prism_string_query__(kind, string, encoding_name)` answers 1
            // for true, 0 for false and -1 for an encoding Prism cannot read.
            "__prism_string_query__" => {
                let query = match arguments.first() {
                    Some(Object::Symbol(kind)) => match kind.as_str().as_ref() {
                        "local" => pm_string_query_local,
                        "constant" => pm_string_query_constant,
                        _ => pm_string_query_method_name,
                    },
                    _ => pm_string_query_method_name,
                };
                let text = string_bytes(arguments.get(1), position)?;
                let encoding = string_bytes(arguments.get(2), position)?;
                let encoding = std::ffi::CString::new(encoding).unwrap_or_default();
                let answer = unsafe { query(text.as_ptr(), text.len(), encoding.as_ptr()) };
                Ok(Object::Int(i64::from(answer)))
            }
            _ => Ok(Object::Nil),
        }
    }
}

/// What MRI's parser is told about code it reads: where it starts, which
/// command-line switches wrap it, whether it is the main script, and the
/// locals of the scopes around it.
pub struct PrismReading<'a> {
    pub start_line: i32,
    pub command_line: &'a str,
    pub main_script: bool,
    pub partial_script: bool,
    pub scopes: Option<&'a [String]>,
}

/// The options prism reads, laid out the way its Ruby library's
/// `dump_options` lays them out, for Ruby 4.0.
fn serialized_options(reading: &PrismReading) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(0u32.to_ne_bytes());
    bytes.extend(reading.start_line.to_ne_bytes());
    bytes.extend(0u32.to_ne_bytes());
    bytes.push(0);
    let switches = reading
        .command_line
        .chars()
        .fold(0u8, |value, switch| match switch {
            'a' => value | 0b000001,
            'e' => value | 0b000010,
            'l' => value | 0b000100,
            'n' => value | 0b001000,
            'p' => value | 0b010000,
            'x' => value | 0b100000,
            _ => value,
        });
    bytes.push(switches);
    // Ruby 4.0's grammar.
    bytes.push(3);
    bytes.push(0);
    bytes.push(u8::from(reading.main_script));
    bytes.push(u8::from(reading.partial_script));
    bytes.push(0);
    match reading.scopes {
        Some(locals) => {
            bytes.extend(1u32.to_ne_bytes());
            bytes.extend((locals.len() as u32).to_ne_bytes());
            // Every kind of argument forwarding is taken as available, so a
            // scope that has one is never refused for using it.
            bytes.push(0x0f);
            for name in locals {
                bytes.extend((name.len() as u32).to_ne_bytes());
                bytes.extend(name.as_bytes());
            }
        }
        None => bytes.extend(0u32.to_ne_bytes()),
    }
    bytes
}

/// Whether MRI's parser refuses `source` read as `reading` says.
pub fn prism_refuses(source: &str, reading: &PrismReading) -> bool {
    let options = serialized_options(reading);
    let bytes = crate::file_loader::escaped_source_bytes(source)
        .unwrap_or_else(|| source.as_bytes().to_vec());
    // SAFETY: the source and the options live for the call, which reads
    // them and keeps neither.
    !unsafe { pm_parse_success_p(bytes.as_ptr(), bytes.len(), options_pointer(&options)) }
}

impl VirtualMachine {
    /// Whether MRI's parser refuses the main script `source`, wrapped by the
    /// command-line switches `command_line` names, such as `n` for `-n`.
    pub fn prism_refuses_program(&self, source: &str, command_line: &str) -> bool {
        prism_refuses(
            source,
            &PrismReading {
                start_line: 1,
                command_line,
                main_script: true,
                partial_script: false,
                scopes: None,
            },
        )
    }

    /// The SyntaxError message MRI gives for a program it refuses, laid out
    /// over prism under `path`, or None when prism accepts it.
    pub fn program_syntax_message(
        &mut self,
        source: &str,
        path: &str,
        start_line: i64,
    ) -> Option<String> {
        let main = self.globals().get("__main__").unwrap_or(Object::Nil);
        let report = self
            .send_to_object(
                main,
                "__syntax_error_report__",
                vec![
                    Object::string(source.to_string()),
                    Object::string(path.to_string()),
                    Object::Int(start_line),
                    Object::array(Vec::new()),
                ],
                Position::default(),
            )
            .ok()?;
        let Object::Array(pair) = report else {
            return None;
        };
        let pair = pair.borrow().clone();
        match pair.as_slice() {
            [Object::String(class), Object::String(message)]
                if &*class.as_str() == "SyntaxError" =>
            {
                Some(message.as_str().to_string())
            }
            _ => None,
        }
    }
}
