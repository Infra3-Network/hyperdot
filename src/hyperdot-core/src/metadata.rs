// pub struct MetadataConfig {
//     pub name: String,
//     pub network: Option<String>,
//     pub path: Option<String>,
//     pub generate: bool,
// }

// impl TryFrom<&Path> for Vec<MetadataConfig> {
//     type Error = anyhow::Error;
//     fn try_from(p: &Path) -> Result<Self, Self::Error> {
//         if p.extension().is_none() {
//             return Err(anyhow::anyhow!("path extension invalid"));
//         }

//         // let yaml_ostr = OsStr::new("yaml");
//         let ext = p.extension().unwrap().to_str().unwrap();
//         match ext {
//             "json" => {
//                 let file = std::fs::File::open(p)?;
//                 let reader = std::io::BufReader::new(file);
//                 match serde_json::from_reader(reader) {
//                     Err(err) => Err(anyhow::anyhow!("{}", err)),
//                     Ok(cl) => Ok(cl),
//                 }
//             }
//             _ => return Err(anyhow::anyhow!("{}: path extension unsupport", ext)),
//         }
//     }
// }
