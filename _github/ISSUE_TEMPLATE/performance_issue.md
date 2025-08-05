---
name: Performance issue
about: Report a performance problem with vexy_json
title: '[PERFORMANCE] '
labels: performance
assignees: ''
---

## ⚡ Performance Issue

**Describe the performance problem**
A clear and concise description of the performance issue you're experiencing.

**Performance Impact**
- [ ] Slow parsing (takes more than expected time)
- [ ] High memory usage
- [ ] Browser freezing/unresponsive
- [ ] Large bundle size
- [ ] Slow loading times

**Input Characteristics**
Please describe the input that causes the performance issue:
- **Input size**: [e.g. 1MB, 10MB, 100KB]
- **Input structure**: [e.g. deeply nested objects, large arrays, many comments]
- **Input complexity**: [e.g. simple flat object, complex nested structure]

**Sample Input** (if possible)
If you can share a sample of the problematic input (anonymized if needed):
```json
{
  "sample": "input that causes performance issues"
}
```

**Performance Measurements**
If you have measurements, please share them:
- **Parse time**: [e.g. 5 seconds, 30 seconds]
- **Memory usage**: [e.g. 500MB, 2GB]
- **Browser**: [e.g. Chrome 120 on macOS]

**Expected Performance**
What performance would you expect for this input?
- **Expected parse time**: [e.g. under 1 second]
- **Expected memory usage**: [e.g. under 100MB]

**Environment**
- **Platform**: [e.g. CLI, Web Tool, Library]
- **Version**: [e.g. 1.1.0]
- **OS**: [e.g. Windows 10, macOS 12, Ubuntu 20.04]
- **Browser** (if web tool): [e.g. Chrome 120, Firefox 115]
- **Hardware**: [e.g. 8GB RAM, M1 MacBook, Intel i7]

**Parser Options**
Which parser options were enabled:
- [ ] Comments
- [ ] Trailing Commas
- [ ] Unquoted Keys
- [ ] Single Quotes
- [ ] Implicit Top Level
- [ ] Newline as Comma

**Comparison**
If you've compared with other JSON parsers, please share the results:
- **Other parser**: [e.g. JSON.parse(), serde_json]
- **Other parser time**: [e.g. 100ms]
- **vexy_json time**: [e.g. 5000ms]

**Additional context**
Add any other context about the performance issue here.

---
*This issue was created using the vexy_json issue template. Performance issues help us optimize the parser for real-world use cases.*