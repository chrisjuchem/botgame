// TODO: fix `crate` references

/// Usage
/// make_enum! {
///     #[derive(Clone, Debug)]
///     pub enum Message {
///         Msg1,
///         Msg2,
///         Msg3,
///     }
/// }
#[macro_export]
macro_rules! make_enum {
    {$(#[$attr:meta])* $vis:vis enum $ety:ident {$($v:tt),* $(,)?}} => {
        crate::macros::make_enum!{@ $(#[$attr])* $vis $ety, $($v $v),*}
    };
    {@ $(#[$attr:meta])* $vis:vis $ety:ident, $($v:ident $vty:ty),*} => {
        $(#[$attr])*
        $vis enum $ety {
            $(#[allow(non_camel_case_types)] $v($vty)),*
        }

        // impl std::fmt::Debug for $ety {
        //     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        //         f.write_fmt(format_args!("{}: {:?}",  stringify!($ety), match self {
        //             $($ety::$v(inner) => inner as &dyn std::fmt::Debug),*
        //         }))
        //     }
        // }

        $(impl std::convert::TryFrom<$ety> for $vty {
            type Error = ();
            fn try_from(e: $ety) -> Result<Self, Self::Error> {
                match e {
                    $ety::$v(inner) => Ok(inner),
                    _ => Err(()),
                }
            }
        })*

        $(impl std::convert::From<$vty> for $ety {
            fn from(v: $vty) -> $ety {
                $ety::$v(v)
            }
        })*
    };
}
pub(crate) use make_enum;

// #[macro_export]
// macro_rules! each_variant {
// {$self:ident, $_as:ty, $_fn:ident, $ety:ident {$($v:tt),* $(,)?}} => {
//         match $self {
//             $($ety::$v(inner) => <$v as $_as>::$_fn(inner)),*
//         }
//     }
// }
// pub(crate) use each_variant;
