Title: issues with getting multitasking to work (Rust).

Findings:

- Running with any level higher than opt 0 fixes (or hides) the problem.
- Running with only one task does not show any problems.
- Instead of saving the context and restoring, re-creating the context each time 
- Removing the debug_println!'s fixes (or hides) the problem.
- The stack overwrites variables above the stack's declaration.