#[allow(dead_code)]

pub struct StringPadder {
    current_max: u32,
}

impl StringPadder {
    pub fn new() -> Self {
        Self { current_max: 0 }
    }

    pub fn add_pad(&mut self, value: impl Into<String>) -> String {
        let value = value.into();
        let current = value.chars().count() as u32;
        let index = self.current_max.saturating_sub(current);

        let ws = &(0..index).into_iter().fold(String::new(), |a, _| a + " ");
        if self.current_max < current {
            self.current_max = current;
        }
        value + ws
    }

    pub fn reset(&mut self) {
        self.current_max = 0;
    }
}

#[cfg(test)]
mod test {
    use super::StringPadder;

    #[test]
    pub fn no_padding() {
        let mut sp = StringPadder::new();
        let value = "foobar".to_string();
        let value = sp.add_pad(value);
        assert_eq!("foobar", value);
    }

    #[test]
    pub fn should_pad_by_one() {
        let mut sp = StringPadder::new();
        let value = "foobar1";
        let value = sp.add_pad(value);
        assert_eq!("foobar1", value);

        let value = "foobar";
        let value = sp.add_pad(value);
        assert_eq!("foobar ", value);
    }

    #[test]
    pub fn should_reset() {
        let mut sp = StringPadder::new();
        let value = "foobar123";
        let value = sp.add_pad(value);
        assert_eq!("foobar123", value);

        let value = "foobar";
        let value = sp.add_pad(value);
        assert_eq!("foobar   ", value);

        sp.reset();

        let value = "foobar";
        let value = sp.add_pad(value);
        assert_eq!("foobar", value);
    }
}
