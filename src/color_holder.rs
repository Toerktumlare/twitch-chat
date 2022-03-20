use std::{any::type_name, collections::HashMap};

use crate::{color_gen, log::get_logger};

pub struct ColorCache {
    cache: HashMap<String, (u8, u8, u8)>,
}

impl ColorCache {
    pub fn new() -> Self {
        ColorCache {
            cache: HashMap::new(),
        }
    }

    pub fn get(&mut self, username: &str) -> (u8, u8, u8) {
        let log = get_logger();
        log.debug(
            format!("fetching color for: {username}"),
            type_name::<ColorCache>(),
        );
        match self.cache.get(username) {
            Some(value) => *value,
            None => {
                log.debug(
                    format!("no color information found for: {username}"),
                    type_name::<ColorCache>(),
                );

                let color = color_gen::get_color();

                log.debug(
                    format!("generating and storing color: {:?}", color),
                    type_name::<ColorCache>(),
                );

                self.cache.insert(username.to_string(), color);
                color
            }
        }
    }
}
