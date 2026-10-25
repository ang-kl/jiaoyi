// `sqlx::migrate!` embeds the migrations at compile time. Without this, a
// changed migration with no other change to the crate leaves the old one
// compiled in (found 25-10-2026: a restored migration kept running mutated).
fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
