fn main() {
    cc::Build::new()
        .include("../vendor/swisseph")
        .define("NOT_WINDOWS", None)
        .opt_level(2)
        .warnings(false)
        .file("../vendor/swisseph/swedate.c")
        .file("../vendor/swisseph/swehouse.c")
        .file("../vendor/swisseph/swejpl.c")
        .file("../vendor/swisseph/swemmoon.c")
        .file("../vendor/swisseph/swemplan.c")
        .file("../vendor/swisseph/sweph.c")
        .file("../vendor/swisseph/swephlib.c")
        .compile("swe");
}
