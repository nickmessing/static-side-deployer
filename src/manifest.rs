use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    Public,
    Private,
}

#[derive(Deserialize)]
pub struct Manifest {
    pub scope: Scope,
    pub domain: String,
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
