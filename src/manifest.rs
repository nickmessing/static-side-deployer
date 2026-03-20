use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Public,
    Private,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub scope: Scope,
    pub domain: String,
    #[serde(default)]
    pub skip_caddy_config: bool,
}

impl Manifest {
    pub fn full_domain(&self) -> String {
        let suffix = match self.scope {
            Scope::Public => "nickmessing.com",
            Scope::Private => "internal",
        };
        format!("{}.{suffix}", self.domain)
    }
}
