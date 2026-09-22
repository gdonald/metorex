// What a method or block was written to take.

use super::*;

impl Parameter {
    /// Create a new simple parameter (no default, not variadic/keyword/block)
    pub fn simple(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: false,
            is_keyword: false,
            is_named_keyword: false,
            is_block: false,
            position,
        }
    }

    /// Create a new parameter with a default value
    pub fn with_default(name: String, default_value: Expression, position: Position) -> Self {
        Parameter {
            name,
            default_value: Some(default_value),
            is_variadic: false,
            is_keyword: false,
            is_named_keyword: false,
            is_block: false,
            position,
        }
    }

    /// Create a new variadic parameter (*args)
    pub fn variadic(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: true,
            is_keyword: false,
            is_named_keyword: false,
            is_block: false,
            position,
        }
    }

    /// Create a new keyword parameter (**kwargs)
    pub fn keyword(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: false,
            is_keyword: true,
            is_named_keyword: false,
            is_block: false,
            position,
        }
    }

    /// Create a new named keyword parameter (`name:` or `name: default`)
    pub fn named_keyword(
        name: String,
        default_value: Option<Expression>,
        position: Position,
    ) -> Self {
        Parameter {
            name,
            default_value,
            is_variadic: false,
            is_keyword: false,
            is_named_keyword: true,
            is_block: false,
            position,
        }
    }

    /// Create a new block parameter (&block)
    pub fn block(name: String, position: Position) -> Self {
        Parameter {
            name,
            default_value: None,
            is_variadic: false,
            is_keyword: false,
            is_named_keyword: false,
            is_block: true,
            position,
        }
    }

    /// Check if this is a simple parameter (no default, not variadic/keyword/block)
    pub fn is_simple(&self) -> bool {
        self.default_value.is_none()
            && !self.is_variadic
            && !self.is_keyword
            && !self.is_named_keyword
            && !self.is_block
    }

    /// Check if this parameter has a default value
    pub fn has_default(&self) -> bool {
        self.default_value.is_some()
    }
}
