macro_rules! map {
    ( $( $k: expr => $v: expr ),* ) => {
        {
            #[allow(unused_mut)]
            let mut m = std::collections::HashMap::new();
            $(
                m.insert($k, $v);
            )*
            m
        }
    };
}
