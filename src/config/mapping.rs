use crate::{
    proc::{CommandContext, Runnable},
    state::Mode,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use viks::{Key, Keymap};

#[derive(Deserialize, Serialize)]
pub(super) struct KeymapConfig {
    pub(super) normal: Option<NormalMaps>,
    pub(super) visual: Option<VisualMaps>,
}

#[derive(Deserialize, Serialize)]
pub(super) struct NormalMaps(pub(super) BTreeMap<String, String>);

#[derive(Deserialize, Serialize)]
pub(super) struct VisualMaps(pub(super) BTreeMap<String, String>);

pub struct KeymapRegistry {
    map: HashMap<(Mode, String), Box<dyn Runnable>>,
}

impl KeymapRegistry {
    pub(super) fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub(super) fn register<R: Runnable + 'static>(
        &mut self,
        mode: Mode,
        map: viks::Result<Keymap>,
        cmd: R,
    ) {
        self.map.insert(
            (
                mode,
                map.expect("Invalid mapping found: KeymapRegistry")
                    .to_string(),
            ),
            Box::new(cmd),
        );
    }

    fn split_to_maps(&self, mode: Mode, keys: Vec<Key>) -> Vec<Result<Vec<Key>, Vec<Key>>> {
        let maps = &self.map;
        let mut res = vec![];
        let mut prenum_buf = vec![];
        let mut buf = vec![];

        for key in keys.into_iter() {
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
                Ok(keys) => {
                    let prenum = &keys
                        .iter()
                        .take_while(|key| key.to_string().chars().all(char::is_numeric))
                        .map(ToString::to_string)
                        .collect::<String>()
                        .parse::<usize>()
                        .ok();
                    let keys = keys
                        .into_iter()
                        .skip_while(|key| key.to_string().chars().all(char::is_numeric))
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
