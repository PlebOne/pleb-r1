# Contributing to Pleb.One

Thank you for your interest in contributing to Pleb.One! This document provides guidelines and information for contributors.

## 🤝 How to Contribute

### Ways to Contribute

- 🐛 **Bug Reports** - Help identify and fix issues
- 💡 **Feature Requests** - Suggest new capabilities  
- 🔧 **Code Contributions** - Submit pull requests
- 📝 **Documentation** - Improve guides and tutorials
- 🎨 **Design** - UI/UX improvements
- 💬 **Community** - Help others in discussions

## 🚀 Getting Started

### 1. Fork and Clone

```bash
# Fork the repository on GitHub
# Then clone your fork
git clone https://github.com/yourusername/pleb-one.git
cd pleb-one
```

### 2. Set Up Development Environment

Follow the [Development Guide](DEVELOPMENT.md) to set up your local environment.

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

## 📋 Development Guidelines

### Code Style

**Rust Code:**
- Use `cargo fmt` to format code
- Run `cargo clippy` for linting
- Follow Rust naming conventions
- Add documentation comments for public APIs

**Frontend Code:**
- Use consistent indentation (2 spaces)
- Follow semantic HTML structure
- Use meaningful variable and function names
- Add comments for complex logic

### Commit Messages

Use conventional commits format:

```
type(scope): description

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

**Examples:**
```
feat(auth): add user registration endpoint
fix(database): resolve SQLite connection issues
docs(readme): update installation instructions
```

### Testing

- Write tests for new features
- Ensure existing tests pass: `cargo test`
- Test both happy path and error cases
- Include integration tests for API endpoints

## 🔍 Pull Request Process

### Before Submitting

1. **Update your branch** with latest main:
   ```bash
   git checkout main
   git pull upstream main
   git checkout your-branch
   git rebase main
   ```

2. **Run tests and checks**:
   ```bash
   cargo test
   cargo fmt --check
   cargo clippy
   ```

3. **Update documentation** if needed

### PR Guidelines

- **Clear Title**: Describe what the PR does
- **Detailed Description**: Explain the changes and why
- **Link Issues**: Reference related issues with "Closes #123"
- **Screenshots**: Include for UI changes
- **Testing**: Describe how you tested the changes

### PR Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Documentation update
- [ ] Refactoring

## Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or marked as such)
```

## 🐛 Bug Reports

### Before Reporting

1. Check if the issue already exists
2. Try to reproduce the issue
3. Test on the latest version

### Bug Report Template

```markdown
**Bug Description**
Clear description of the bug

**Steps to Reproduce**
1. Step one
2. Step two
3. Step three

**Expected Behavior**
What should happen

**Actual Behavior**
What actually happens

**Environment**
- OS: [e.g. Windows 11]
- Rust version: [e.g. 1.70.0]
- Node version: [e.g. 18.17.0]

**Additional Context**
Any other relevant information
```

## 💡 Feature Requests

### Feature Request Template

```markdown
**Feature Description**
Clear description of the proposed feature

**Problem Statement**
What problem does this solve?

**Proposed Solution**
How should this work?

**Alternatives**
Other solutions considered

**Additional Context**
Any other relevant information
```

## 📚 Documentation

### Types of Documentation

- **API Documentation** - Document new endpoints
- **User Guides** - Help users use features
- **Developer Guides** - Help developers contribute
- **Deployment Guides** - Production setup instructions

### Documentation Standards

- Use clear, concise language
- Include practical examples
- Keep documentation updated with code changes
- Use proper markdown formatting

## 🎯 Areas for Contribution

### High Priority

- [ ] **Email Verification System** - User registration verification
- [ ] **Payment Integration** - Stripe/Lightning payment processing  
- [ ] **React Platform Dependencies** - Fix npm dependency conflicts
- [ ] **Docker Configuration** - Complete containerization setup
- [ ] **Testing Coverage** - Comprehensive test suite

### Medium Priority

- [ ] **Admin Dashboard** - User management interface
- [ ] **Metrics Enhancement** - Advanced monitoring capabilities
- [ ] **Mobile Optimization** - Responsive design improvements
- [ ] **Performance Optimization** - Database and API optimization

### Good First Issues

- [ ] **Documentation Improvements** - Fix typos, add examples
- [ ] **Code Cleanup** - Remove unused imports, fix warnings
- [ ] **UI Polish** - Small design improvements
- [ ] **Configuration Enhancement** - Add more environment options

## 🏗️ Architecture Guidelines

### Project Structure

```
pleb-one/
├── services/
│   ├── relay-engine/     # Core Rust relay
│   └── community-web/    # Frontend applications
├── docs/                 # Documentation
├── DEVELOPMENT.md        # Development guide
└── README.md            # Project overview
```

### Component Guidelines

- **Separation of Concerns** - Keep components focused
- **Error Handling** - Comprehensive error management
- **Configuration** - Environment-based settings
- **Logging** - Structured logging throughout
- **Testing** - Unit and integration tests

## 🤖 Automation

### GitHub Actions

We use GitHub Actions for:
- Automated testing on PRs
- Code formatting checks
- Security scanning
- Deployment to staging

### Local Development Tools

Recommended tools:
- **Rust Analyzer** - VS Code extension for Rust
- **Prettier** - Code formatting for frontend
- **GitLens** - Enhanced Git integration
- **Thunder Client** - API testing in VS Code

## 💬 Community

### Communication Channels

- **GitHub Issues** - Bug reports and feature requests
- **GitHub Discussions** - General discussion and questions
- **Discord** - Real-time community chat (coming soon)

### Code of Conduct

We follow the [Contributor Covenant](https://www.contributor-covenant.org/) code of conduct. Be respectful, inclusive, and constructive in all interactions.

## 🏆 Recognition

Contributors will be recognized in:
- README.md contributors section
- Release notes for significant contributions
- Community highlights
- GitHub contributor stats

## 📞 Getting Help

### Development Questions

1. Check existing documentation
2. Search GitHub issues and discussions
3. Ask in GitHub Discussions
4. Join our Discord community

### Mentorship

New contributors can request mentorship for:
- Understanding the codebase
- Choosing good first issues
- Code review guidance
- Development best practices

---

**Thank you for contributing to Pleb.One! Together we're building the future of decentralized infrastructure. 🚀**
