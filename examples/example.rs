extern crate audited;
use audited::*;

pub struct MyOtherStruct {}

mod another_mod {
    pub struct MyCoolStruct {}
}

pub use another_mod::*;

#[audited(
    sig: "895476084D37BC4C9EE5C469EB1BD1178AE631DA7903481123F379AB2A921A0F\
          75FBB06DCA306D9258F08C642788D8514CF531A3DF7F64095B507F4C98713F05",
    timestamp: "Tue, 29 Nov 2022 05:11:31 +0000",
    signed_by: "Sam Johnson <sam@durosoft.com>",
    public: "2193B7E4EE81686E4FE7FA700967A4E142259152265449E5AE2D69B959464317",
    allow_use: true,
    allowed_foreign_paths: [crate::MyOtherStruct, MyCoolStruct],
)]
mod some_mod {
    use crate::*;

    pub struct _MyStruct {}

    mod sub_module {
        pub struct _ThisIsOk {}
    }

    pub type _ThisIsOk = sub_module::_ThisIsOk;

    pub type _MyOtherStruct = crate::MyOtherStruct;
    pub type _MyOtherStruct2 = crate::MyOtherStruct;
    pub const _SOMETHING: MyCoolStruct = MyCoolStruct {};
    pub const _SOMETHING_ELSE: _ThisIsOk = sub_module::_ThisIsOk {};
}

fn main() {
    // #[audited_use("7FA700967A4E142")]
    // use some_mod::MyStruct;
    // let _c: MyStruct = MyStruct {};
    // #[audited_use]
    // use some_mod::MyOtherStruct;
    // let _d: MyOtherStruct = MyOtherStruct {};
    // let _a = 1;
    // let _b = 2;
}
