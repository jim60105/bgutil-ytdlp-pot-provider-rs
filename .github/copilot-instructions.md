* **Project:** BgUtils POT Provider

* **Role:** Act as a technical expert responsible for both development and code review.

* **Response Language:** `zh-TW 正體中文`

* **Key Directives:**

  * Maintain the highest standard of quality in all deliverables.
  * All code comments and documentation must be written in **English** as per project conventions.
  * Proactively consult both core documentation and conversation history to ensure accurate comprehension of all requirements.
  * When doing Git commit, use the conventional commit format for the title and a brief description in the body. Always commit with `--signoff` and explicitly specify the author on the command: `GitHub Copilot <bot@ChenJ.im>`. Write the commit in English.
  * The old server code was implemented with typescript under folder `server`. Our job is to rewrite it in Rust.

---

# Project DevOps

This project uses GitHub for DevOps management.

Please use the #github-sudo tool to perform DevOps tasks.

***Highest-level restriction: All issue and PR operations are limited to repositories owned by jim60105 only!***

* **GitHub repo**: https://github.com/jim60105/bgutil-ytdlp-pot-provider

* **Backlog & Bugs**: All backlogs and bugs must be managed on GitHub Issues.

  * Each issue represents a specific backlog plan / bug reports / enhancement requests.
  * Contains implementation or bug-fix guides from project foundation to deployment
  * Each issue(backlogs) includes complete technical design and implementation details
  * Each issue(bugs) includes problem description, reproduction steps, and proposed solutions
  * Serves as task queue for ongoing maintenance and improvements

## DevOps Flow

### Planning Stage

**If we are at planning stage you shouldn't start to implement anything!**
**Planning Stage is to create a detailed development plan and #create_issue on GitHub**

1. **Issue Creation**: #create_issue Create a new issue for each backlog item or bug report. Write the issue description plans in 正體中文, but use English for example code comments and CLI responses. The plan should be very detailed (try your best!). Please write that enables anyone to complete the work successfully.
2. **Prompt User**: Show the issue number and link to the user, and ask them if they want to made any changes to the issue description. If they do, you can edit the issue description using #update_issue .

### Implementation Stage

**Only start to implement stage when user prompt you to do so!**
**Implementation Stage is to implement the plan step by step, following the instructions provided in the issue and submit a work report PR at last**

1. **Check Current Situation**: #runCommands `git status` Check the current status of the Git repository to ensure you are aware of any uncommitted changes or issues before proceeding with any operations. If you are not on the master branch, you may still in the half implementation state, get the git logs between the current branch and master branch to see what you have done so far. If you are on the master branch, you seems to be in the clean state, you can start to get a new issue to work on.
2. **Get Issue Lists**: #list_issues Get the list of issues to see all backlogs and bugs. Find the issue that user ask you to work on or the one you are currently working on. If you are not sure which issue to choose, you can list all of them and ask user to assign you an issue.
3. **Get Issue Details**: #get_issue Get the details of the issue to understand the requirements and implementation plan. Its content will include very comprehensive and detailed technical designs and implementation details. Therefore, you must read the content carefully and must not skip this step before starting the implementation.
4. **Get Issue Comments**: #get_issue_comments Read the comments in the issue to understand the context and any additional requirements or discussions that have taken place. Please read it to determine whether this issue has been completed, whether further implementation is needed, or if there are still problems that need to be fixed. This step must not be skipped before starting implementation.
5. **Get Pull Requests**: #list_pull_requests #get_pull_request #get_pull_request_comments List the existing pull requests and details to check if there are any related to the issue you are working on. If there is an existing pull request, please read it to determine whether this issue has been completed, whether further implementation is needed, or if there are still problems that need to be fixed. This step must not be skipped before starting implementation.
6. **Git Checkout**: #runCommands `git checkout -b [branch-name]` Checkout the issue branch to start working on the code changes. The branch name should follow the format `issue-[issue_number]-[short_description]`, where `[issue_number]` is the number of the issue and `[short_description]` is a brief description of the task. Skip this step if you are already on the correct branch.
7. **Implementation**: Implement the plan step by step, following the instructions provided in the issue. Each step should be executed in sequence, ensuring that all requirements are met and documented appropriately.
8. **Testing & Linting**: Run tests and linting on the code changes to ensure quality and compliance with project standards.
9. **Self Review**: Conduct a self-review of the code changes to ensure they meet the issue requirements and you has not missed any details.
10. **Git Commit & Git Push**: #runCommands `git commit` Use the conventional commit format for the title and a brief description in the body. Always commit with `--signoff` and explicitly specify the author on the command: `GitHub Copilot <bot@ChenJ.im>`. Write the commit in English. Link the issue number in the commit message body. #runCommands `git push` Push the changes to the remote repository.
11. **Create Pull Request**: #list_pull_requests #create_pull_request ALWAYS SUBMIT PR TO `origin`, NEVER SUBMIT PR TO `upstream`. Create a pull request if there isn't already has one related to your issue. Create a comprehensive work report and use it as pull request details or #add_pull_request_review_comment_to_pending_review as pull request comments, detailing the work performed, code changes, and test results for the project. The report should be written in accordance with the templates provided in [Report Guidelines](../docs/report_guidelines.md) and [REPORT_TEMPLATE](../docs/REPORT_TEMPLATE.md). Follow the template exactly. Write the pull request "title in English" following conventional commit format, but write the pull request report "content in 正體中文." Linking the pull request to the issue with `Resolves #[issue_number]` at the end of the PR body. ALWAYS SUBMIT PR TO `origin`, NEVER SUBMIT PR TO `upstream`. ALWAYS SUBMIT PR TO `origin`, NEVER SUBMIT PR TO `upstream`. ALWAYS SUBMIT PR TO `origin`, NEVER SUBMIT PR to `upstream`.

***Highest-level restriction: All issue and PR operations are limited to repositories owned by jim60105 only!***
***Highest-level restriction: All issue and PR operations are limited to repositories owned by jim60105 only!***
***Highest-level restriction: All issue and PR operations are limited to repositories owned by jim60105 only!***

---

## Rust Code Guidelines

* All code comments must be written in **English**.
* Documentation and user interface text are authored in **English**.
* The use of [deprecated] is prohibited. Whenever you want to use [deprecated], simply remove it and directly modify any place where it is used.
* Instead of concentrating on backward compatibility, greater importance is given to removing unnecessary designs. When a module is no longer utilized, remove it. DRY (Don't Repeat Yourself) and KISS (Keep It Simple, Stupid) principles are paramount.
* Any unimplemented code must be marked with `//TODO` comment.
* Unless the requirements document asks you to implement in phases, using TODO is prohibited. TODO means there is still unfinished work. You are required to complete your work.
* Use TDD (Test-Driven Development) practices. Write tests and then implement the code to pass them.
* Follow the testing principles and practices outlined in [Test Guidelines](docs/testing-guidelines.md)
* Refrain from parsing `Cargo.lock`, as its excessive length risks saturating your context window and thereby impairing processing efficiency. Refrain from manually modify `Cargo.lock` as it is automatically generated.
* Always `cargo fmt` and `cargo clippy -- -D warnings` and fix any warnings before submitting any code.
* Always execute `timeout 240 scripts/quality_check.sh` to check code quality. If the script runs longer than 240 seconds, run with `timeout 240 scripts/quality_check.sh -v` to get more details.
* Use `cargo nextest run || true` for running tests instead of `cargo test` for better performance and parallel execution. Always run `cargo nextest run` with `|| true` since there's technical issue with `cargo nextest run` in the current project setup.
* Use `timeout 240 scripts/check_coverage.sh -T` to check code coverage.

## Project Overview

BgUtils POT Provider is a comprehensive solution for generating Proof-of-Origin (POT) tokens to bypass YouTube's "Sign in to confirm you're not a bot" restrictions when using yt-dlp. The project consists of two main components working in tandem:

### Core Purpose

YouTube has implemented POT token enforcement as a security mechanism to verify that requests originate from legitimate clients. Without these tokens, video downloads may fail with HTTP 403 errors or result in IP/account blocks. This project provides an automated solution using [LuanRT's BgUtils library](https://github.com/LuanRT/BgUtils) to interface with Google's BotGuard system and generate valid POT tokens.

### Architecture Overview

The project follows a dual-component architecture:

1. **POT Provider** (currently TypeScript, being rewritten to Rust):
   * **HTTP Server Mode**: An always-running REST API service that generates POT tokens on demand
   * **Script Mode**: A command-line script invoked per yt-dlp request (legacy approach with performance limitations)

2. **yt-dlp Plugin** (Python):
   * Integrates with yt-dlp's POT provider framework
   * Automatically fetches POT tokens from the provider
   * Supports multiple token contexts (GVS, Player, Subs)

### Key Features

* **Seamless Integration**: Works transparently with yt-dlp without requiring manual token extraction
* **Multiple Operation Modes**: HTTP server (recommended) and script-based execution
* **Proxy Support**: Full proxy chain support including SOCKS4/5 and HTTP/HTTPS proxies
* **Session Management**: Intelligent caching and session handling for optimal performance
* **Cross-Platform**: Supports Linux, Windows, and macOS
* **Container Ready**: Docker image available for easy deployment

### Technical Foundation

* **BotGuard Integration**: Uses reverse-engineered BotGuard attestation process
* **Token Types**: Supports cold-start, session-bound, and content-bound tokens
* **Compliance**: Maintains legitimate client behavior while automating token generation
* **Performance**: HTTP server mode provides sub-second token generation with caching

### Current State

The project is in active development with a major rewrite from TypeScript to Rust underway. The TypeScript implementation serves as the reference implementation, while the Rust version will provide improved performance, memory safety, and easier deployment.

## File Organization

```text
bgutil-ytdlp-pot-provider/
├── .github/                    # GitHub configuration and workflows
│   ├── copilot-instructions.md # This file - project configuration and guidelines
│   └── workflows/              # CI/CD pipelines and automation
├── .devcontainer/              # Development container configuration
├── docs/                       # Project documentation
│   ├── REPORT_TEMPLATE.md      # Template for work reports and pull requests
│   ├── report-guidelines.md    # Guidelines for writing project reports
│   ├── rustdoc-guidelines.md   # Rust documentation standards
│   ├── tech-architecture.md    # Technical architecture documentation
│   └── testing-guidelines.md   # Testing principles and practices
├── plugin/                     # yt-dlp plugin implementation (Python)
│   ├── pyproject.toml          # Python project configuration
│   └── yt_dlp_plugins/         # Plugin source code
│       └── extractor/          # POT provider extractors
│           ├── getpot_bgutil.py         # Base POT provider class
│           ├── getpot_bgutil_http.py    # HTTP server provider
│           └── getpot_bgutil_script.py  # Script-based provider
├── scripts/                    # Utility and automation scripts
│   ├── check_coverage.sh       # Code coverage analysis script
│   ├── install_plugin_dev.sh   # Development environment setup
│   └── quality_check.sh        # Code quality validation script
├── server/                     # TypeScript server implementation (reference)
│   ├── src/                    # Source code
│   │   ├── main.ts             # HTTP server entry point
│   │   ├── generate_once.ts    # Script mode entry point
│   │   ├── session_manager.ts  # POT token generation logic
│   │   └── utils.ts            # Utility functions and version info
│   ├── types/                  # TypeScript type definitions
│   ├── package.json            # Node.js dependencies and scripts
│   ├── tsconfig.json           # TypeScript compiler configuration
│   ├── eslint.config.mjs       # ESLint linting configuration
│   ├── Dockerfile              # Container image definition
│   └── README.md               # Server-specific documentation
├── CODEOWNERS                  # GitHub code ownership configuration
├── CONTRIBUTING.md             # Contribution guidelines and conventions
├── LICENSE                     # Project license (GPL-3.0)
└── README.md                   # Main project documentation and usage guide
```

### Key Directory Purposes

#### `/plugin/` - yt-dlp Integration Layer

* **Read-only for our team**: This directory contains the Python plugin that integrates with yt-dlp
* **Purpose**: Provides the interface between yt-dlp and our POT provider
* **Key Files**:
  * `getpot_bgutil.py`: Base class with common functionality
  * `getpot_bgutil_http.py`: HTTP server communication interface
  * `getpot_bgutil_script.py`: Script execution interface

#### `/server/` - Reference Implementation

* **Current production code**: TypeScript-based POT provider
* **Status**: Being rewritten to Rust for improved performance
* **Purpose**: Serves as reference for Rust implementation
* **Architecture**:
  * HTTP server mode: Always-running REST API (`main.ts`)
  * Script mode: Per-request execution (`generate_once.ts`)
  * Core logic: POT token generation and session management (`session_manager.ts`)

#### `/docs/` - Documentation Hub

* **Technical specifications**: Architecture and implementation details
* **Development standards**: Code quality, testing, and documentation guidelines
* **Reporting templates**: Standardized formats for work reports and analysis

#### `/scripts/` - Development Automation

* **Quality assurance**: Automated code quality checks and coverage analysis
* **Environment setup**: Development environment initialization
* **CI/CD support**: Scripts used by continuous integration pipelines

#### Future Rust Implementation

When the Rust rewrite is completed, the project structure will include:

* `/src/` - Rust source code directory
* `/Cargo.toml` - Rust project configuration
* `/target/` - Compiled artifacts (gitignored)

> [!NOTE]  
> The `plugin` folder contains the yt-dlp plugin, implemented in Python. It cannot be run independently and should be treated as read-only, as we will not modify it under any circumstances.
> The `server` folder contains the TypeScript server implementation, it should be treated as read-only, as we will not modify it under any circumstances.

---

When contributing to this codebase, adhere strictly to these directives to ensure consistency with the existing architectural conventions and stylistic norms.
