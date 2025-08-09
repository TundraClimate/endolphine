use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::Path};

pub struct Exec {
    pub cmd: String,
    pub args: Vec<String>,
}

impl<'de> Deserialize<'de> for Exec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct StrOrVec;

        impl<'de> Visitor<'de> for StrOrVec {
            type Value = Exec;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "a value is available to string or [string]")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                let v = v.trim();

                if v.is_empty() {
                    return Err(serde::de::Error::custom("empty literal aren't available"));
                }

                Ok(Exec {
                    cmd: v.to_string(),
                    args: vec![],
                })
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut args = vec![];

                let Some(cmd) = seq.next_element::<String>()? else {
                    return Err(serde::de::Error::custom("first element aren't available"));
                };

                if cmd.is_empty() {
                    return Err(serde::de::Error::custom("empty command aren't available"));
                }

                while let Some(element) = seq.next_element::<String>()? {
                    let element = element.trim();

                    if element.is_empty() {
                        return Err(serde::de::Error::custom("empty literal aren't available"));
                    }

                    args.push(element.to_string());
                }

                Ok(Exec { cmd, args })
            }
        }

        deserializer.deserialize_any(StrOrVec)
    }
}

impl Serialize for Exec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        if self.args.is_empty() {
            serializer.serialize_str(&self.cmd)
        } else {
            let mut seq = serializer.serialize_seq(Some(self.args.len() + 1))?;

            seq.serialize_element(&self.cmd)?;

            for elem in self.args.iter() {
                seq.serialize_element(elem)?;
            }

            seq.end()
        }
    }
}

#[derive(Serialize)]
pub(super) struct EditConfig(pub(super) BTreeMap<String, HijackInfo>);

impl Default for EditConfig {
    fn default() -> Self {
        let mut ed = BTreeMap::new();

        ed.insert(
            "default".to_string(),
            HijackInfo {
                cmd: Exec {
                    cmd: "vim".to_string(),
                    args: vec![],
                },
                hijack: true,
            },
        );

        Self(ed)
    }
}

#[derive(Deserialize, Serialize)]
pub struct HijackInfo {
    pub cmd: Exec,
    pub hijack: bool,
}

impl<'de> Deserialize<'de> for EditConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Visitor;

        struct MapChecker;

        impl<'de> Visitor<'de> for MapChecker {
            type Value = EditConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "mapping keys are available the . prefix or 'default'"
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut data = BTreeMap::new();
                let mut default_flag = false;

                while let Some((key, value)) = map.next_entry::<String, HijackInfo>()? {
                    if !key.starts_with(".") && &key != "default" {
                        return Err(serde::de::Error::custom(
                            "mapping keys are available the . prefix or 'default'",
                        ));
                    }

                    if &key == "default" {
                        default_flag = true;
                    }

                    data.insert(key, value);
                }

                if !default_flag {
                    return Err(serde::de::Error::custom("default mapping not initialized"));
                }

                Ok(EditConfig(data))
            }
        }

        deserializer.deserialize_map(MapChecker)
    }
}

pub struct HijackMapping(BTreeMap<String, HijackInfo>);

impl HijackMapping {
    pub(super) fn new(config: EditConfig) -> Self {
        Self(config.0)
    }

    pub fn get(&self, file: &Path) -> Option<&HijackInfo> {
        if !file.is_file() {
            return None;
        }

        let extension = file.extension()?.to_str()?;

        self.0.get(&format!(".{extension}"))
    }

    pub fn default_ed(&self) -> &HijackInfo {
        self.0
            .get("default")
            .expect("default mapping not initialized")
    }
}
