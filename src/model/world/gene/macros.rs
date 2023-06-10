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
                    $( $var: random_range($lower, $upper) ),*
                }
            }
            #[allow(unused_assignments)]
            fn mutate_one(&mut self) {
                let mut threshold = 0.0;
                let delta = 1.0 / count_fields!($( $var ),*) as f32;
                let rand = random::<f32>();
                $( if rand < threshold {
                    self.$var = random_range($lower, $upper);
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
                        self.$var += ($upper - $lower) / 10 as $ty;
                    } else {
                        self.$var -= ($upper - $lower) / 10 as $ty;
                    }
                    self.$var= self.$var.clamp($lower, $upper);
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
