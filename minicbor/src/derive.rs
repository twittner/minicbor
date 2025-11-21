#[cfg(feature = "std")]
#[doc(hidden)]
#[macro_export]
macro_rules! __minicbor_cfg {
    (
        'std {$($std:tt)*}
        'alloc {$($alloc:tt)*}
        'otherwise {$($otherwise:tt)*}
    ) => {
        $($std)*
    };
}

#[cfg(all(feature = "alloc", not(feature = "std")))]
#[doc(hidden)]
#[macro_export]
macro_rules! __minicbor_cfg {
    (
        'std {$($std:tt)*}
        'alloc {$($alloc:tt)*}
        'otherwise {$($otherwise:tt)*}
    ) => {
        $($alloc)*
    };
}

#[cfg(all(not(feature = "alloc"), not(feature = "std")))]
#[doc(hidden)]
#[macro_export]
macro_rules! __minicbor_cfg {
    (
        'std {$($std:tt)*}
        'alloc {$($alloc:tt)*}
        'otherwise {$($otherwise:tt)*}
    ) => {
        $($otherwise)*
    };
}
