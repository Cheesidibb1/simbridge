# Contributing to SimBridge

Thank you for your interest in contributing to SimBridge! This document provides guidelines for contributing to the project.

## Code of Conduct

- Be respectful and inclusive
- Provide constructive feedback
- Focus on what is best for the community
- Show empathy towards other community members

## Getting Started

1. Fork the repository
2. Clone your fork:
```bash
git clone https://github.com/yourusername/simbridge.git
cd simbridge
```

3. Add the upstream remote:
```bash
git remote add upstream https://github.com/original/simbridge.git
```

4. Create a feature branch:
```bash
git checkout -b feature/my-feature
```

## Development Workflow

### Making Changes

1. Make your changes following the project's coding standards
2. Write tests for your changes
3. Ensure all tests pass
4. Update documentation if needed

### Committing Changes

Follow the commit message format:

```
<type>: <subject>

<body>

<footer>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

Example:
```
feat: add GPS streaming support

Implemented GPS streaming from companion app to simulator.
Added location service and message handling.

Closes #123
```

### Pull Requests

1. Push your changes:
```bash
git push origin feature/my-feature
```

2. Create a pull request on GitHub
3. Fill out the PR template
4. Wait for review

## Coding Standards

### Rust

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Document public APIs with `///`
- Keep functions small and focused

### Flutter/Dart

- Use `flutter format` for formatting
- Use `flutter analyze` for static analysis
- Follow Effective Dart guidelines
- Use meaningful variable names
- Prefer const constructors

### General

- Write clear, self-documenting code
- Add comments for complex logic
- Keep functions under 50 lines
- Use meaningful names for variables and functions
- Follow DRY (Don't Repeat Yourself) principle

## Testing

### Rust Tests

```bash
cd shared
cargo test

cd ../server
cargo test
```

### Flutter Tests

```bash
cd companion
flutter test

cd ../desktop
flutter test
```

### Writing Tests

#### Rust

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(2 + 2, 4);
    }

    #[tokio::test]
    async fn test_async_example() {
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

#### Flutter

```dart
void main() {
  testWidgets('Example test', (WidgetTester tester) async {
    await tester.pumpWidget(MyWidget());
    expect(find.text('Hello'), findsOneWidget);
  });

  group('Group of tests', () {
    test('Test 1', () {
      expect(1 + 1, equals(2));
    });

    test('Test 2', () {
      expect(2 * 2, equals(4));
    });
  });
}
```

## Documentation

- Update README.md for user-facing changes
- Update DEVELOPER.md for developer-facing changes
- Update API.md for API changes
- Add inline documentation for complex code
- Keep documentation in sync with code changes

## Issue Reporting

When reporting issues, include:

1. Clear description of the problem
2. Steps to reproduce
3. Expected behavior
4. Actual behavior
5. Environment details (OS, versions, etc.)
6. Relevant logs or error messages
7. Screenshots if applicable

## Feature Requests

When requesting features:

1. Describe the use case
2. Explain why it's needed
3. Suggest a possible implementation
4. Consider if it fits the project scope

## Review Process

1. All PRs require at least one approval
2. CI must pass
3. Code must be properly formatted
4. Tests must be added/updated
5. Documentation must be updated

## Release Process

Releases are versioned using semantic versioning (MAJOR.MINOR.PATCH):

- MAJOR: Breaking changes
- MINOR: New features (backwards compatible)
- PATCH: Bug fixes (backwards compatible)

## Questions?

Feel free to open an issue for questions or discussion.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
