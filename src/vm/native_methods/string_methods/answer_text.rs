// An answer built one piece at a time, each piece carrying the
// encoding it was written in.

use super::*;

/// An answer built one piece at a time, each piece written in an encoding of
/// its own. An answer holding only ASCII so far takes on the encoding of the
/// first piece that holds more, and a later piece that cannot be read in that
/// encoding is a CompatibilityError.
pub(crate) struct AnswerText {
    pub(crate) built: String,
    pub(crate) encoding: String,
    pub(crate) holds_only_ascii: bool,
}

impl AnswerText {
    /// An answer with nothing in it yet, written in the encoding the text it
    /// is built from was written in.
    pub(crate) fn new(encoding: &str) -> Self {
        Self {
            built: String::new(),
            encoding: encoding.to_string(),
            holds_only_ascii: true,
        }
    }

    /// What has been built so far.
    pub(crate) fn text(self) -> String {
        self.built
    }

    /// The encoding the pieces settled on.
    pub(crate) fn encoding(&self) -> String {
        self.encoding.clone()
    }

    pub(crate) fn add(
        &mut self,
        piece: &str,
        encoding: &str,
        position: Position,
    ) -> Result<(), MetorexError> {
        if piece.is_empty() {
            return Ok(());
        }
        // Text in an encoding that spells ASCII over several bytes never
        // joins text in another, however little either of them holds.
        if self.encoding != encoding
            && (!crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(encoding)
                || !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                    &self.encoding,
                ))
        {
            return Err(crate::vm::errors::simple_exception(
                "Encoding::CompatibilityError",
                &format!(
                    "incompatible character encodings: {} and {}",
                    self.encoding, encoding
                ),
                position,
            ));
        }
        if !piece.is_ascii() {
            if self.holds_only_ascii {
                self.encoding = encoding.to_string();
                self.holds_only_ascii = false;
            } else if self.encoding != encoding {
                return Err(crate::vm::errors::simple_exception(
                    "Encoding::CompatibilityError",
                    &format!(
                        "incompatible character encodings: {} and {}",
                        self.encoding, encoding
                    ),
                    position,
                ));
            }
        } else if self.holds_only_ascii
            && !crate::vm::native_methods::class_methods::encoding_reads_alongside_ascii(
                &self.encoding,
            )
        {
            self.encoding = encoding.to_string();
        }
        self.built.push_str(piece);
        Ok(())
    }
}
