> Give a bulleted list of things a code reviewer (regardless of implementation language) should attend to during a thorough, expert code review. Be sure to include things that are easy for people to forget.

## Code Annotation, Indexing, Cross-reference

Cataloging major types. Descriptions include

- Ownership and lifecycle
- Purpose and function
- Subordinate types, structs, and enums

(A *subordinate type* is a type that exists solely in the service and support of the primary type.)



## Upon Code Modification 

### Comments and documentation

Do the modifications to the code require adjustments to the comments within the source code or to the written documentation (whereever it may be)?

LLM: Analyze the code change with respect to the code comments and the written documentation.

- Determine if the code comments are still accurate and relevant.
- Determine if additional code commentary is indicated.
- Likewise for the library/API documentation. In addition:
  - If the modification adds features/functionality, make sure it's documented.
  - Determine if existing examples still cover the typical use-cases. 



1. **Code Complexity and Readability Analysis**
   - Calculate and report on the complexity of functions (e.g., cyclomatic complexity).
   - Identify code segments that are overly complex or hard to read and suggest simplifications or refactoring.
2. **Consistency Checks**
   - Verify naming conventions for variables, functions, classes, and other identifiers against the project's style guide.
   - Check for consistency in code formatting and style throughout the codebase.
3. **Dependency Analysis**
   - Map out and visualize the dependencies between different parts of the codebase.
   - Identify unused or redundant dependencies to reduce the code footprint.
4. **Security and Vulnerability Assessment**
   - Perform static code analysis to detect common security vulnerabilities (e.g., SQL injection, buffer overflow).
   - Highlight sections of code that may be prone to security risks.
5. **Performance Analysis**
   - Identify potential performance bottlenecks, such as inefficient loops or unoptimized algorithms.
   - Suggest more efficient algorithms or data structures where applicable.
6. **Test Coverage Analysis**
   - Assess the test coverage of the codebase and identify critical paths or functions that are under-tested.
   - Generate suggestions for additional tests or testing strategies.
7. **Code Smells and Anti-patterns Detection**
   - Identify and highlight code smells and anti-patterns that could lead to maintenance issues or technical debt.
8. **Internationalization and Localization Review**
   - Check if the codebase is prepared for internationalization (i18n) and localization (l10n), identifying hard-coded strings or locale-specific elements.
9. **Documentation and Comment Quality**
   - Evaluate the quality and completeness of inline comments and external documentation.
   - Suggest areas where comments are missing, unclear, or could be improved.
10. **Legal and Licensing Compliance**
    - Scan the codebase for compliance with licensing requirements and third-party copyrights.
    - Identify any potential legal issues with code or dependency usage.
11. **Historical Code Review**
    - Analyze the evolution of the codebase over time to understand historical decisions and their impact on the current state.
    - Identify patterns or changes in the code that have led to bugs or issues in the past.
12. **Automated Refactoring Suggestions**
    - Provide automated refactoring suggestions to improve code quality, structure, or maintainability without changing its functionality.
13. **Resource Leak and Management**
    - Detect potential resource leaks (e.g., memory, file handles, network connections) and suggest fixes.
14. **Concurrency Issues Analysis**
    - Identify and highlight potential concurrency issues like deadlocks, race conditions, or improper synchronization.
15. **Integration with Development Tools**
    - Suggest integrations or plugins that could improve the development workflow, based on the codebase analysis.

These suggestions aim to cover a broad range of aspects that are crucial for a thorough code review process, leveraging the capabilities of an LLM to handle tasks that are often time-consuming, error-prone, or overlooked by human reviewers.