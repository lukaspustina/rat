error_chain! {
    errors {
        NoCommandSpecified {
            description("no command specified")
            display("no command specified")
        }

        NoSubcommandSpecified(module_name: String) {
            description("no sub command specified")
            display("no sub command for module {} specified", module_name)
        }

        ModuleFailed(module_name: String) {
            description("module failed")
            display("executing module {} failed", module_name)
        }
    }
}