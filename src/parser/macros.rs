macro_rules! takes {
    ($pairs:expr, $amount:literal) => {{
        use itertools::Itertools;
        $pairs.clone().into_inner().take($amount).collect_tuple().unwrap()
    }};
}

macro_rules! has {
    ($pairs:expr, $typed:literal) => {
        $pairs.into_inner().any(|a| format!("{:?}", a.as_rule()) == $typed.to_string())
    };
}

macro_rules! ident {
    ($pair:expr) => {
        match format!("{:?}", $pair.as_rule()) == "identifier".to_string() {
            true => Ok($pair.as_str().to_string()),
            false => Err(anyhow!("Failed to get  ident")),
        }
    };
}

macro_rules! string {
    ($pair: expr) => {
        $pair.clone().into_inner().next().unwrap().as_str()
    };
}
