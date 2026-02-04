---
trigger: always_on
---

# Tool Usage Guidelines

## String Search Strategy

### 1. Primary Tool: `grep_search`
Use `grep_search` as the first choice for finding string patterns, function definitions, or references across the codebase.
- **Pros**: Fast, structured JSON output, native tool integration.
- **Best for**: Finding code symbols, specific text snippets, or checking for existence of patterns in standard-sized files.
- **Usage**:
  - `Query`: The string or regex to search for.
  - `SearchPath`: Directory or specific file.
  - `Includes`: Filter by file extension (e.g., `["*.rs"]`).

### 2. Large Files & Complex Queries: `run_command`
When `grep_search` is insufficient (e.g., extremely large files > 1MB or > 10,000 lines) or when you need to leverage system-specific tools for performance.

**Windows (cmd/batch preferred):**
Use `findstr` for reliable, line-numbered output.
```cmd
findstr /n /i /c:"search_term" path\to\file
```
- `/n`: Prints line numbers (CRITICAL for referencing location).
- `/i`: Case-insensitive.
- `/c:"..."`: Literal string search (avoids regex parsing issues for simple strings).

**PowerShell:**
```powershell
Select-String -Pattern "search_term" -Path "path\to\file" -Context 0,5
```

### 3. Workflow for Large Files
1. **Locate**: Use `grep_search` or `run_command` to find the specific line numbers.
2. **View**: Use `view_file` with `StartLine` and `EndLine` (e.g., +/- 50 lines around the match).
   - **Do not** read entire large files (>2000 lines) at once.
   - **Do not** use `view_file` to "search" by scrolling.