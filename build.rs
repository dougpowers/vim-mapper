fn main() {
    #[cfg(target_family = "windows")]
    embed_resource::compile("embed_icon.rc");
}