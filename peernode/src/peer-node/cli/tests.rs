#[cfg(test)]
mod tests {
    use crate::cli;

    #[test]
    fn test_clap_cli() {
        // Simulate command line arguments
        let args = vec!["market", "set", "localhost:50051"];

        // Call your CLI function with the arguments
        let result = cli().get_matches_from(args);

        // Assert the expected output
        //assert_eq!(result, Some(_));
    }
}
