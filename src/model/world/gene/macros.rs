#[macro_export]
macro_rules! make_gene_struct {
    ( $vis:vis $name:ident { $( $var:ident: $ty:ty = $lower:tt..$upper:tt  ),* , } ) => {
        #[derive(Debug, Clone)]
        $vis struct $name {
            $(pub $var: $ty ),*
        }
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
                let random = random::<f32>();
                $( if random < threshold {
                    self.$var = random_range($lower, $upper);
                    return;
                } else {
                    threshold += delta;
                } )*
            }
            #[allow(dead_code)]
            fn breed(&self, other: &Self) -> Self {
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
