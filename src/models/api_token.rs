pub struct Rule {
  pub v1: Option<V1Rule>
}

pub struct V1Rule {
  pipeline: bool,
  models: HashMap<bool>,
  notebooks: HashMap<bool>,
}


