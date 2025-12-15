fn main() {
    if let Err(error) = project_mapper_runtime::entrypoint::run_main() {
        panic!("{:#}", error);
    }
}
