//! The `rb_c*`, `rb_m*` and `rb_e*` globals naming core classes and modules,
//! set before an extension's `Init` function runs. One whose class metorex
//! does not define stays nil.

use super::handles::{QNIL, Value, to_value};
use crate::lexer::Position;
use crate::object::Object;
use crate::vm::VirtualMachine;

/// MRI's `fatal`, the exception that ends the interpreter, which Ruby code
/// cannot name as a constant.
#[unsafe(no_mangle)]
pub static mut rb_eFatal: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mKernel: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mComparable: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mEnumerable: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mErrno: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mFileTest: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mGC: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mMath: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mProcess: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mWaitReadable: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_mWaitWritable: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cBasicObject: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cObject: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cArray: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cBinding: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cClass: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cComplex: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cDir: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cEncoding: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cEnumerator: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cFalseClass: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cFile: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cFloat: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cHash: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cIO: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cInteger: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cMatch: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cMethod: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cModule: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cNilClass: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cNumeric: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cProc: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cRandom: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cRange: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cRational: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cRegexp: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cSet: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cStat: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cString: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cStruct: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cSymbol: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cThread: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cTime: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cTrueClass: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_cUnboundMethod: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eException: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eStandardError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSystemExit: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eInterrupt: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSignal: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eArgError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eEOFError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eIndexError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eStopIteration: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eKeyError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eRangeError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eIOError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eRuntimeError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eFrozenError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSecurityError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSystemCallError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eThreadError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eTypeError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eZeroDivError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNotImpError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNoMemError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNoMethodError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eFloatDomainError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eLocalJumpError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSysStackError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eRegexpError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eEncodingError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eEncCompatError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNoMatchingPatternError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNoMatchingPatternKeyError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eScriptError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eNameError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eSyntaxError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eLoadError: Value = QNIL;
#[unsafe(no_mangle)]
pub static mut rb_eMathDomainError: Value = QNIL;

impl VirtualMachine {
    /// Points each global at the class or module it names.
    pub(super) fn publish_core_classes(&mut self) {
        let Some(object_class) = self.globals().get("Object") else {
            return;
        };
        // SAFETY: the interpreter runs C extensions on one thread, and these
        // are written before any C code that reads them runs.
        let globals: [(*mut Value, &str); 79] = [
            (&raw mut rb_mKernel, "Kernel"),
            (&raw mut rb_mComparable, "Comparable"),
            (&raw mut rb_mEnumerable, "Enumerable"),
            (&raw mut rb_mErrno, "Errno"),
            (&raw mut rb_mFileTest, "FileTest"),
            (&raw mut rb_mGC, "GC"),
            (&raw mut rb_mMath, "Math"),
            (&raw mut rb_mProcess, "Process"),
            (&raw mut rb_mWaitReadable, "IO::WaitReadable"),
            (&raw mut rb_mWaitWritable, "IO::WaitWritable"),
            (&raw mut rb_cBasicObject, "BasicObject"),
            (&raw mut rb_cObject, "Object"),
            (&raw mut rb_cArray, "Array"),
            (&raw mut rb_cBinding, "Binding"),
            (&raw mut rb_cClass, "Class"),
            (&raw mut rb_cComplex, "Complex"),
            (&raw mut rb_cDir, "Dir"),
            (&raw mut rb_cEncoding, "Encoding"),
            (&raw mut rb_cEnumerator, "Enumerator"),
            (&raw mut rb_cFalseClass, "FalseClass"),
            (&raw mut rb_cFile, "File"),
            (&raw mut rb_cFloat, "Float"),
            (&raw mut rb_cHash, "Hash"),
            (&raw mut rb_cIO, "IO"),
            (&raw mut rb_cInteger, "Integer"),
            (&raw mut rb_cMatch, "MatchData"),
            (&raw mut rb_cMethod, "Method"),
            (&raw mut rb_cModule, "Module"),
            (&raw mut rb_cNilClass, "NilClass"),
            (&raw mut rb_cNumeric, "Numeric"),
            (&raw mut rb_cProc, "Proc"),
            (&raw mut rb_cRandom, "Random"),
            (&raw mut rb_cRange, "Range"),
            (&raw mut rb_cRational, "Rational"),
            (&raw mut rb_cRegexp, "Regexp"),
            (&raw mut rb_cSet, "Set"),
            (&raw mut rb_cStat, "File::Stat"),
            (&raw mut rb_cString, "String"),
            (&raw mut rb_cStruct, "Struct"),
            (&raw mut rb_cSymbol, "Symbol"),
            (&raw mut rb_cThread, "Thread"),
            (&raw mut rb_cTime, "Time"),
            (&raw mut rb_cTrueClass, "TrueClass"),
            (&raw mut rb_cUnboundMethod, "UnboundMethod"),
            (&raw mut rb_eException, "Exception"),
            (&raw mut rb_eStandardError, "StandardError"),
            (&raw mut rb_eSystemExit, "SystemExit"),
            (&raw mut rb_eInterrupt, "Interrupt"),
            (&raw mut rb_eSignal, "SignalException"),
            (&raw mut rb_eArgError, "ArgumentError"),
            (&raw mut rb_eEOFError, "EOFError"),
            (&raw mut rb_eIndexError, "IndexError"),
            (&raw mut rb_eStopIteration, "StopIteration"),
            (&raw mut rb_eKeyError, "KeyError"),
            (&raw mut rb_eRangeError, "RangeError"),
            (&raw mut rb_eIOError, "IOError"),
            (&raw mut rb_eRuntimeError, "RuntimeError"),
            (&raw mut rb_eFrozenError, "FrozenError"),
            (&raw mut rb_eSecurityError, "SecurityError"),
            (&raw mut rb_eSystemCallError, "SystemCallError"),
            (&raw mut rb_eThreadError, "ThreadError"),
            (&raw mut rb_eTypeError, "TypeError"),
            (&raw mut rb_eZeroDivError, "ZeroDivisionError"),
            (&raw mut rb_eNotImpError, "NotImplementedError"),
            (&raw mut rb_eNoMemError, "NoMemoryError"),
            (&raw mut rb_eNoMethodError, "NoMethodError"),
            (&raw mut rb_eFloatDomainError, "FloatDomainError"),
            (&raw mut rb_eLocalJumpError, "LocalJumpError"),
            (&raw mut rb_eSysStackError, "SystemStackError"),
            (&raw mut rb_eRegexpError, "RegexpError"),
            (&raw mut rb_eEncodingError, "EncodingError"),
            (&raw mut rb_eEncCompatError, "Encoding::CompatibilityError"),
            (
                &raw mut rb_eNoMatchingPatternError,
                "NoMatchingPatternError",
            ),
            (
                &raw mut rb_eNoMatchingPatternKeyError,
                "NoMatchingPatternKeyError",
            ),
            (&raw mut rb_eScriptError, "ScriptError"),
            (&raw mut rb_eNameError, "NameError"),
            (&raw mut rb_eSyntaxError, "SyntaxError"),
            (&raw mut rb_eLoadError, "LoadError"),
            (&raw mut rb_eMathDomainError, "Math::DomainError"),
        ];
        // SAFETY: as above.
        unsafe { rb_eFatal = to_value(&self.fatal_class()) };
        for (global, path) in globals {
            let found = self.send_to_object(
                object_class.clone(),
                "const_get",
                vec![Object::string(path)],
                Position::default(),
            );
            let value = found.map_or(QNIL, |class| to_value(&class));
            // SAFETY: as above.
            unsafe { *global = value };
        }
    }
}

thread_local! {
    static FATAL: std::cell::OnceCell<Object> = const { std::cell::OnceCell::new() };
}

impl VirtualMachine {
    /// The `fatal` exception class, made once as a subclass of Exception.
    fn fatal_class(&mut self) -> Object {
        let Some(Object::Class(exception)) = self.globals().get("Exception") else {
            return Object::Nil;
        };
        FATAL.with(|held| {
            held.get_or_init(|| {
                let made = crate::class::Class::new("fatal", Some(std::rc::Rc::clone(&exception)));
                exception.add_subclass(&made);
                Object::Class(made)
            })
            .clone()
        })
    }
}
