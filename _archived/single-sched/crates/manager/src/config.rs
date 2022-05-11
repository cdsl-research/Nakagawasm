use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub executor: Executor,
    pub wasi: Wasi,
    pub threshold: Option<u64>,
    pub outdir: String,
}

#[derive(Debug, Deserialize)]
pub struct Executor {
    pub path: String,
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
        let md = MapDir {host: "hoge".into(), guest: "fuga".into()};
        assert_eq!("host = 'hoge'\nguest = 'fuga'\n", &toml::to_string_pretty(&md).unwrap());
    }

    #[test]
    fn test_mapdir_toml_de() {
        let expected = MapDir {host: "hoge".into(), guest: "fuga".into()};

        let text = "host = 'hoge'\nguest = 'fuga'\n";
        assert_eq!(expected, toml::from_str::<MapDir>(&text).unwrap());
        
        #[derive(Debug, Deserialize)]
        struct Test {
            inner: MapDir,
        }

        let text = "inner = { host = 'hoge', guest = 'fuga' }";
        assert_eq!(expected , toml::from_str::<Test>(&text).unwrap().inner);
    }
}
