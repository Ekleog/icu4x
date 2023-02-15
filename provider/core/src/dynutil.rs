pub trait UpcastDataPayload<M>
where
    M: crate::DataMarker,
    Self: Sized + crate::DataMarker,
{
    fn upcast(other: crate::DataPayload<M>) -> crate::DataPayload<Self>{loop{}}
}
macro_rules! impl_dynamic_data_provider {
    (:ty, $arms:tt, , ) => {
        ::!;
        ::!(
            ,
            ,
            $($rest),+
        );
    };
    (, { $( = : => $struct_m:ty),+, }, ) => {
        impl ::<> for
        {
            fn  -> <
                ::<>,
                ::,
            > {
                match . {
                    $(
                         => {
                            let : ::<> =
                                ::::<>::?;
                            (:: {
                                : .,
                                : ..(|| {
                                    ::dynutil::UpcastDataPayload::<$struct_m>::upcast(p)
                                }),
                            })
                        }
                    )+,
                    _ =>
                }
            }
        }
    };
    (, [ $($struct_m:ident),+, ], ) => {
        impl ::<> for
        {
            fn  -> <
                ::<>,
                ::,
            > {
                #!
                $(
                    const $struct_m: $crate::DataKeyHash = $struct_m::KEY.hashed();
                )+
                match .
            }
        }
    };
}
