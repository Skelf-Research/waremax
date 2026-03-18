# Documentation Guide

Standards for writing documentation.

---

## Documentation Types

### Code Documentation

Rustdoc comments in source code:

```rust
/// Calculate distance between two points.
///
/// # Arguments
///
/// * `x1`, `y1` - First point coordinates
/// * `x2`, `y2` - Second point coordinates
///
/// # Returns
///
/// Euclidean distance as `f64`
pub fn distance(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    // ...
}
```

### User Documentation

MkDocs files in `documentation/docs/`:

- Getting started guides
- Configuration reference
- Tutorials

### API Documentation

Generated from code:

```bash
cargo doc --open
```

---

## Writing Guidelines

### Be Concise

```markdown
<!-- Good -->
Run the simulation:
`waremax run scenario.yaml`

<!-- Avoid -->
In order to run the simulation, you need to execute
the following command in your terminal...
```

### Use Active Voice

```markdown
<!-- Good -->
The scheduler processes events in time order.

<!-- Avoid -->
Events are processed by the scheduler in time order.
```

### Show, Don't Tell

```markdown
<!-- Good -->
Configure robot count:
```yaml
robots:
  count: 10
```

<!-- Avoid -->
To configure the robot count, you need to set the count
field under the robots section.
```

---

## Markdown Style

### Headers

```markdown
# Page Title (H1 - one per page)

## Major Section (H2)

### Subsection (H3)

#### Minor heading (H4 - use sparingly)
```

### Code Blocks

Always specify language:

````markdown
```yaml
robots:
  count: 10
```

```rust
fn example() {
    println!("Hello");
}
```

```bash
waremax run scenario.yaml
```
````

### Lists

```markdown
Unordered:
- First item
- Second item
  - Nested item

Ordered:
1. First step
2. Second step
3. Third step
```

### Tables

```markdown
| Column 1 | Column 2 | Column 3 |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
| Value 4  | Value 5  | Value 6  |
```

---

## Page Structure

### Standard Page

```markdown
# Page Title

Brief introduction (1-2 sentences).

---

## Overview

What this page covers.

---

## Main Content

### Section 1

Content...

### Section 2

Content...

---

## Examples

Practical examples.

---

## Related

- [Related Page 1](link1.md)
- [Related Page 2](link2.md)
```

### Tutorial Page

```markdown
# Tutorial Title

Brief description.

---

## Goal

What reader will accomplish.

**Time**: X minutes

---

## Prerequisites

What reader needs to know.

---

## Step 1: First Step

Instructions...

## Step 2: Second Step

Instructions...

---

## Summary

What was covered.

---

## Next Steps

Where to go next.
```

### Reference Page

```markdown
# Component Reference

## Overview

What this component does.

---

## Configuration

### Option 1

Description.

```yaml
option1: value
```

### Option 2

Description.

```yaml
option2: value
```

---

## Examples

Complete examples.

---

## Related

Links to related pages.
```

---

## Admonitions

Use for important callouts:

```markdown
!!! note
    Additional information.

!!! warning
    Something to be careful about.

!!! danger
    Critical warning.

!!! tip
    Helpful suggestion.

!!! example
    Example content.
```

---

## Links

### Internal Links

```markdown
[Link text](relative/path.md)
[Link to section](path.md#section-anchor)
```

### External Links

```markdown
[External link](https://example.com)
```

### Cross-References

```markdown
See [Configuration](../configuration/index.md) for details.
```

---

## Images

### Placement

Put images in `docs/assets/images/`:

```markdown
![Alt text](../assets/images/diagram.png)
```

### Diagrams

Prefer text-based diagrams (ASCII, Mermaid):

````markdown
```mermaid
graph LR
    A[Start] --> B[Process]
    B --> C[End]
```
````

---

## API Documentation

### Module Documentation

```rust
//! # Module Name
//!
//! Brief description.
//!
//! ## Overview
//!
//! What this module provides.
//!
//! ## Examples
//!
//! ```rust
//! use crate::module;
//! // Example code
//! ```
```

### Function Documentation

```rust
/// Brief description.
///
/// Longer description if needed.
///
/// # Arguments
///
/// * `arg1` - Description
/// * `arg2` - Description
///
/// # Returns
///
/// Description of return value.
///
/// # Errors
///
/// Describes when errors occur.
///
/// # Examples
///
/// ```rust
/// let result = function(arg1, arg2);
/// ```
///
/// # Panics
///
/// Describes panic conditions (if any).
pub fn function(arg1: Type, arg2: Type) -> Result<T, E> {
    // ...
}
```

---

## Building Documentation

### MkDocs

```bash
cd documentation
mkdocs serve  # Local preview
mkdocs build  # Build static site
```

### Rustdoc

```bash
cargo doc --open
cargo doc --no-deps  # Skip dependencies
```

---

## Review Checklist

Before submitting documentation:

- [ ] Spelling and grammar checked
- [ ] Code examples tested
- [ ] Links verified
- [ ] Consistent formatting
- [ ] Appropriate for target audience
- [ ] No broken images

---

## Related

- [Code Style](code-style.md)
- [Testing Guide](testing.md)
