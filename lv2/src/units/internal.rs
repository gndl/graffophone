
macro_rules! unit_list {
    () => {};
    ($(($camelcase:ident, $sysname:ident, $typename:ty));+) => {
        $(unit_list! {$camelcase, $sysname, $typename})+
    };
    ($camelcase:ident,$sysname:ident,$typename:ty) => {
        #[derive(Copy, Clone, PartialEq, Debug)]
        #[repr(transparent)]
        pub struct $camelcase(pub $typename);

        unsafe impl ::core::uri::UriBound for $camelcase {
            const URI: &'static [u8] = ::lv2_sys::$sysname;
        }
        impl ::units::Unit for $camelcase {}
    };
}
