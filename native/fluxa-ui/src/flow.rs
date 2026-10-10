pub(crate) struct Column {
    cursor: f32,
}

impl Column {
    pub(crate) fn from(top: f32) -> Self {
        Self { cursor: top }
    }

    pub(crate) fn place(&mut self, height: f32) -> f32 {
        let top = self.cursor;
        self.cursor += height;
        top
    }

    pub(crate) fn end(&self) -> f32 {
        self.cursor
    }
}
