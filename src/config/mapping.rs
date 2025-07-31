use crate::{
    proc::{CommandContext, Runnable},
    state::Mode,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use viks::{Key, Keymap};

#[derive(Deserialize, Serialize)]
pub(super) struct KeymapConfig {
    pub(super) normal: Option<UserDefinedMaps>,
    pub(super) visual: Option<UserDefinedMaps>,
    pub(super) menu: Option<UserDefinedMaps>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct UserDefinedMaps(BTreeMap<Keymap, Keymap>);

impl UserDefinedMaps {
    pub(super) fn collect_maps(&self) -> Vec<(Keymap, Keymap)> {
        self.0.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }
}

pub struct KeymapRegistry {
    map: HashMap<(Mode, String), Box<dyn Runnable>>,
}

impl KeymapRegistry {
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(super) fn register<R: Runnable + 'static>(&mut self, mode: Mode, map: Keymap, cmd: R) {
        self.map.insert((mode, map.to_string()), Box::new(cmd));
    }

    pub(super) fn register_raw<R: Runnable + 'static>(
        &mut self,
        mode: Mode,
        map: viks::Result<Keymap>,
        cmd: R,
    ) {
        self.register(
            mode,
            map.unwrap_or_else(|e| {
                panic!("Invalid mapping found: '{}' is {}", e.format(), e.cause())
            }),
            cmd,
        );
    }

    fn split_to_maps(&self, mode: Mode, keys: Vec<Key>) -> Vec<Result<Vec<Key>, Vec<Key>>> {
        let maps = &self.map;
        let mut res = vec![];
        let mut prenum_buf = vec![];
        let mut buf = vec![];

        for key in keys.into_iter() {
            if mode == Mode::Input {
                if maps.contains_key(&(mode, key.to_string())) {
                    res.push(Ok(vec![key]));
                }

                continue;
            }

            let as_str = key.to_string();
            let mut chars = as_str.chars();

            if buf.is_empty() && chars.all(char::is_numeric) {
                prenum_buf.push(key);

                continue;
            }

            buf.push(key);

            if maps.contains_key(&(mode, Keymap::from(buf.clone()).to_string())) {
                let mut keymap = vec![];

                keymap.append(&mut prenum_buf);
                keymap.append(&mut buf);

                res.push(Ok(keymap));
            }
        }

        if !prenum_buf.is_empty() || !buf.is_empty() {
            let mut keymap = vec![];

            keymap.append(&mut prenum_buf);
            keymap.append(&mut buf);

            res.push(Err(keymap));
        }

        res
    }

    fn get(&self, mode: Mode, keys: Keymap) -> Option<&dyn Runnable> {
        self.map.get(&(mode, keys.to_string())).map(|cmd| &**cmd)
    }

    pub fn has_similar_map(&self, mode: Mode, keys: Keymap) -> bool {
        self.map
            .keys()
            .filter_map(|(m, map)| (mode == *m).then_some(map))
            .filter_map(|map| Keymap::new(map).ok())
            .filter(|map| map.as_vec().len() >= keys.as_vec().len())
            .any(|map| {
                let keys = keys.as_vec();

                map.as_vec()[..keys.len()] == keys[..]
            })
    }

    #[allow(clippy::type_complexity)]
    pub fn eval_keys(
        &self,
        mode: Mode,
        keys: Vec<Key>,
    ) -> Vec<Result<(&dyn Runnable, CommandContext), Vec<Key>>> {
        let parsed = self.split_to_maps(mode, keys);

        parsed
            .into_iter()
            .map(|map| match map {
                Ok(keys) if mode == Mode::Input => {
                    let cmd = self
                        .get(mode, Keymap::from(keys))
                        .expect("Incorrect code found");

                    Ok((cmd, CommandContext::new(None)))
                }
                Ok(keys) => {
                    let prenum = &keys
                        .iter()
                        .take_while(|key| key.is_digit())
                        .map(ToString::to_string)
                        .collect::<String>()
                        .parse::<usize>()
                        .ok();
                    let keys = keys
                        .into_iter()
                        .skip_while(|key| key.is_digit())
                        .collect::<Vec<_>>();

                    let item = (
                        self.get(mode, Keymap::from(keys))
                            .expect("Incorrect code found"),
                        CommandContext::new(*prenum),
                    );

                    Ok(item)
                }
                Err(keys) => Err(keys),
            })
            .collect::<Vec<_>>()
    }
}
