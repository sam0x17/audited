extern crate audited;
use audited::*;

#[audited(
    sig: "895476084D37BC4C9EE5C469EB1BD1178AE631DA7903481123F379AB2A921A0F\
          75FBB06DCA306D9258F08C642788D8514CF531A3DF7F64095B507F4C98713F05",
    timestamp: "Tue, 29 Nov 2022 05:11:31 +0000",
    signed_by: "Sam Johnson <sam@durosoft.com>",
    public: "2193B7E4EE81686E4FE7FA700967A4E142259152265449E5AE2D69B959464317"
)]
fn main() {
    let _a = 1;
    let _b = 2;
}
