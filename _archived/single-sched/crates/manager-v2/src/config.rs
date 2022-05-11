use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub wasi: Wasi,
    pub threshold: Option<u32>,
    pub outdir: String,
}

impl Config {
    pub async fn from_toml_file(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(path).await?;
        let config = toml::from_str(s.as_str())?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct Wasi {
    pub path: String,
    #[serde(rename = "mapdir")]
    pub mapdirs: Vec<MapDir>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapDir {
    pub host: String,
    pub guest: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapdir_toml_ser() {
        let md = MapDir {
            host: "hoge".into(),
            guest: "fuga".into(),
        };
        assert_eq!(
            "host = 'hoge'\nguest = 'fuga'\n",
            &toml::to_string_pretty(&md).unwrap()
        );
    }

    #[test]
    fn test_mapdir_toml_de() {
        let expected = MapDir {
            host: "hoge".into(),
            guest: "fuga".into(),
        };

        let text = "host = 'hoge'\nguest = 'fuga'\n";
        assert_eq!(expected, toml::from_str::<MapDir>(&text).unwrap());

        #[derive(Debug, Deserialize)]
        struct Test {
            inner: MapDir,
        }

        let text = "inner = { host = 'hoge', guest = 'fuga' }";
        assert_eq!(expected, toml::from_str::<Test>(&text).unwrap().inner);
    }
}
