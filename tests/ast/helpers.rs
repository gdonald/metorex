use metorex::lexer::Position;

pub(super) fn pos(line: usize, column: usize) -> Position {
    Position::new(line, column, 0)
}
