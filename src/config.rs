use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Station {
    pub line_ref: String,
    pub stop_point_ref: String,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub api_key: String,
    pub stations: Vec<Station>,
}

impl ::std::default::Default for Config {
    // See https://data.iledefrance-mobilites.fr/explore/dataset/referentiel-des-lignes/information/?disjunctive.transportmode&disjunctive.transportsubmode&disjunctive.operatorname&disjunctive.networkname
    // See https://data.iledefrance-mobilites.fr/explore/dataset/arrets/table/
    fn default() -> Self {
        Self {
            api_key: "".into(),
            stations: vec![
                Station {
                    line_ref: "C01378".into(),
                    stop_point_ref: "A463226".into(),
                    name: "Michel Bizot (8) (Balard)".into(),
                },
                Station {
                    line_ref: "C02251".into(),
                    stop_point_ref: "A23512".into(),
                    name: "Wattignies - Gravelle (77) (Gare de Lyon)".into(),
                },
                Station {
                    line_ref: "C01119".into(),
                    stop_point_ref: "A23512".into(),
                    name: "Wattignies - Gravelle (87) (Invalides)".into(),
                },
                Station {
                    line_ref: "C01743".into(),
                    stop_point_ref: "A473907".into(),
                    name: "Luxembourg (RER B) (Robinson • Saint-Rémy-lès-Chevreuse)".into(),
                },
            ],
        }
    }
}

pub fn init() -> Config {
    let cfg: Config = confy::load("ontake/nezumi-p", "config").unwrap();
    confy::store("ontake/nezumi-p", "config", cfg.clone()).unwrap();
    cfg
}
