// What a hash and its entries read back as.

use super::*;

impl VirtualMachine {
    /// One `key => value` entry as `inspect` shows it. A Symbol key reads as
    /// `name: value`, the way Ruby prints one.
    pub(crate) fn render_pair(
        &mut self,
        key: &Object,
        value: &Object,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        let (rendered_value, _) = self.rendered_with_encoding(value, position)?;
        if let Object::Symbol(name) = key {
            let spelled = Object::String(Rc::clone(name));
            let (quoted, writing) = self.rendered_with_encoding(&spelled, position)?;
            // A symbol that is a plain name prints without quotes, as long as
            // the encoding the answer is written in has room for every
            // character of it.
            // The name is read through the encoding it carries, since that is
            // what says which characters it holds.
            let spelled_name = crate::vm::native_methods::name_text(name);
            let plain = is_plain_symbol_name(&spelled_name)
                && (name.holds_bytes() || quoted == format!("\"{}\"", name.as_str()));
            let shown = if plain {
                name.as_str().to_string()
            } else {
                quoted
            };
            // A name carried as the bytes an encoding spells it with is
            // written in that encoding, whatever `inspect` tagged its text
            // with.
            let writing = if name.holds_bytes() {
                name.encoding_name()
            } else {
                writing
            };
            return Ok((format!("{}: {}", shown, rendered_value), writing));
        }
        let (rendered_key, writing) = self.rendered_with_encoding(key, position)?;
        Ok((format!("{} => {}", rendered_key, rendered_value), writing))
    }

    /// What `inspect` answered for an object, and the encoding it wrote the
    /// answer in.
    fn rendered_with_encoding(
        &mut self,
        obj: &Object,
        position: Position,
    ) -> Result<(String, String), MetorexError> {
        let rendered = self.inspected_object(obj, position)?;
        Ok(match &rendered {
            Object::String(text) => (text.to_string(), text.encoding_name()),
            other => (format!("{}", other), "US-ASCII".to_string()),
        })
    }
}

/// Text tagged with an encoding, where one was named. A value of any other
/// kind is answered as it stands.
/// A String frozen where it stands, which is how the environment hands back
/// what it holds.
pub(crate) fn frozen_text(value: Object) -> Object {
    let Object::String(text) = &value else {
        return value;
    };
    let copy = crate::object::StringValue::with_encoding(text.to_text(), text.encoding_name());
    copy.freeze();
    Object::String(std::rc::Rc::new(copy))
}

pub(crate) fn retagged(value: Object, named: &Option<String>) -> Object {
    let (Object::String(text), Some(named)) = (&value, named) else {
        return value;
    };
    // Bytes that do not read as the encoding the environment is named in
    // stand for themselves, which is what Ruby hands back for them.
    let named = match named.as_str() {
        "US-ASCII" if !text.as_str().is_ascii() => "ASCII-8BIT".to_string(),
        other => other.to_string(),
    };
    Object::String(std::rc::Rc::new(crate::object::StringValue::with_encoding(
        text.to_text(),
        named,
    )))
}

/// Whether a symbol is named plainly enough to print as a hash key without
/// quotes: a letter or underscore, then letters, digits and underscores, and
/// at most one `?` or `!` to close it.
fn is_plain_symbol_name(name: &str) -> bool {
    let body = name.strip_suffix(['?', '!']).unwrap_or(name);
    let mut letters = body.chars();
    // Every character outside ASCII stands in a name, which is what lets a
    // symbol written in another script print without quotes.
    letters
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_' || !first.is_ascii())
        && letters.all(|letter| letter.is_alphanumeric() || letter == '_' || !letter.is_ascii())
}
