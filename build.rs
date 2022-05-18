fn main() {
    println!("cargo:rerun-if-changed=client/src");
    std::process::Command::new("sh")
        .arg("-c")
        .arg("cd client && yarn run build");
}
