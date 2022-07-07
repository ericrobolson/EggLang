# Decisions
- Make both a CLI and a library/c library for this
- Use Rust as a core
- Easily embeddable on all platforms
- Bottom up development. Take a user story, then focus on building up what you need from the bottom (lowest component) up. E.g. need to get a webpage's contents? Start with the HTTP client, then the response parser, then the application logic after the others are done.