#[macro_export]
macro_rules! make_gene_struct {
    ( $vis:vis $name:ident { $( $var:ident: $ty:ty = $lower:tt..$upper:tt  ),* , } ) => {
        #[derive(Debug, Clone)]
        #[allow(clippy::identity_op)]
        #[allow(unused_parens)]
        $vis struct $name {
            $(pub $var: $ty ),*
        }
        #[allow(clippy::identity_op)]
        #[allow(unused_parens)]
        impl $name {
            fn random() -> Self {
                Self {
                    $( $var: random_range($lower as $ty, $upper as $ty) ),*
                }
            }
            #[allow(unused_assignments)]
            fn mutate_one(&mut self) {
                let mut threshold = 0.0;
                let delta = 1.0 / count_fields!($( $var ),*) as f32;
                let rand = random::<f32>();
                $( if rand < threshold {
                    self.$var = random_range($lower as $ty, $upper as $ty);
                    return;
                } else {
                    threshold += delta;
                } )*
            }
            #[allow(unused_assignments)]
            fn mutate_one_gradual(&mut self) {
                let mut threshold = 0.0;
                let delta = 1.0 / count_fields!($( $var ),*) as f32;
                let rand = random::<f32>();
                $( if rand < threshold {
                    if random() {
                        self.$var += ($upper as $ty - $lower as $ty) / 10 as $ty;
                    } else {
                        self.$var -= ($upper as $ty - $lower as $ty) / 10 as $ty;
                    }
                    self.$var= self.$var.clamp($lower as $ty, $upper as $ty);
                    // make range exclusive
                    if self.$var == $upper as $ty {
                        self.$var = random_range($lower as $ty, $upper as $ty);
                    }
                    return;
                } else {
                    threshold += delta;
                } )*
            }
            #[allow(dead_code)]
            fn cross_over(&self, other: &Self) -> Self {
                Self {
                    $( $var: if random() { self.$var } else { other.$var } ),*
                }
            }
        }
        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "(")?;
                $( write!(f, "{}: {}, ", stringify!($var), self.$var)?; )*
                write!(f, ")")
            }
        }
    };
}

pub(crate) use make_gene_struct;

macro_rules! replace_expr {
    ($_t:tt $sub:expr) => {
        $sub
    };
}
pub(crate) use replace_expr;

macro_rules! count_fields {
    ($( $var:tt ),*  ) => {
        {<[()]>::len(&[$(replace_expr!($var ())),*])}
    };
}
pub(crate) use count_fields;
