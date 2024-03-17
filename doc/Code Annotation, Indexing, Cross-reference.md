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