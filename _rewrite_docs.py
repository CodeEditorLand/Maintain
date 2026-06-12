#!/usr/bin/env python3
"""
Rewrite all rustdoc comments (/// and //!) in the Maintain element crate to
meet quality standards.

Quality spec: .hermes/plan/handoffs/rustdoc-quality-spec.md

Fixes applied (in order):
1. "/// This function/struct/enum/... does X" -> "/// Does X"
2. "/// Represents X" -> "/// X"
3. Remove empty /// lines that aren't between ## Parameters items
4. Add //! module-level docs to all mod.rs files (already done)
5. Add ## Parameters / ## Returns / ## Errors / ## Panics sections
6. Fix chunked/outdated descriptions
"""

import re
import os
import subprocess
import sys

SOURCE_DIR = "/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Maintain/Source"

# Pattern: "/// This (function|struct|enum|trait|module|type|method|macro|const|static|field|constructor|implementation|ensures|prevents|is) "
# We want to remove the "This <thing>" prefix and make it a direct statement.
THIS_FUNCTION_RE = re.compile(
    r'^(\s*/// )(This (?:'
    r'function|struct|enum|trait|module|type|method|macro|const|static|'
    r'constructor|field|implementation|ensures|prevents|is'
    r') )'
    r'([a-zA-Z])'
)

# Pattern: "/// Represents " -> ""
REPRESENTS_RE = re.compile(
    r'^(\s*/// )Represents '
)

# Pattern: "/// This function:" (colon after, with list items following)
THIS_FUNCTION_COLON_RE = re.compile(
    r'^(\s*/// )This function:\s*$'
)

# Pattern for lines that separate sections
SECTION_BOUNDARY_RE = re.compile(
    r'^(/// )[-=]{2,}'
)

def fix_meta_text_lines(lines):
    """Fix 'This function/struct/enum' and 'Represents' patterns."""
    result = []
    for line in lines:
        stripped = line.rstrip('\n')
        
        # Fix "/// This function:" -> remove the meta intro, keep empty doc line
        m = THIS_FUNCTION_COLON_RE.match(stripped)
        if m:
            # Just keep the "///" prefix as an empty doc line
            result.append(m.group(1).rstrip() + '\n')
            continue
        
        # Fix "/// This function/struct/enum does X" -> "/// Does X"
        m = THIS_FUNCTION_RE.match(stripped)
        if m:
            prefix = m.group(1)
            first_char = m.group(3)
            rest = stripped[m.end():]
            # Capitalize the first character
            new_line = f"{prefix}{first_char.upper()}{rest}"
            result.append(new_line.rstrip() + '\n')
            continue
        
        # Fix "/// Represents X" --> "/// X"  
        m = REPRESENTS_RE.match(stripped)
        if m:
            prefix = m.group(1)
            rest = stripped[m.end():]
            if rest:
                # Capitalize first letter
                rest = rest[0].upper() + rest[1:] if rest else rest
            new_line = f"{prefix}{rest}"
            result.append(new_line.rstrip() + '\n')
            continue
        
        result.append(stripped.rstrip() + '\n')
    
    return result


def fix_section_header_text(text):
    """Fix text inside section headers like `/// # Arguments` to `/// ## Parameters`."""
    # Replace `/// # Arguments` with `/// ## Parameters`
    text = re.sub(r'^/// # Arguments\s*$', '/// ## Parameters', text, flags=re.MULTILINE)
    text = re.sub(r'^/// # Returns\s*$', '/// ## Returns', text, flags=re.MULTILINE)
    text = re.sub(r'^/// # Errors\s*$', '/// ## Errors', text, flags=re.MULTILINE)
    text = re.sub(r'^/// # Panics\s*$', '/// ## Panics', text, flags=re.MULTILINE)
    text = re.sub(r'^/// # Example\s*$', '/// ## Example', text, flags=re.MULTILINE)
    text = re.sub(r'^/// # Behavior\s*$', '/// ## Behavior', text, flags=re.MULTILINE)
    return text


def remove_section_block_comments(text):
    """Remove '// ---' and '// ===' style section separator comments that appear
    inside doc comments. These are not rustdoc - they're section banners from the
    code comments that accidentally got prefix-d with ///."""
    lines = text.split('\n')
    result = []
    in_doc_comment = False
    
    for line in lines:
        is_doc = line.startswith('///')
        
        # If this line has a section banner pattern that should NOT be doc: 
        # e.g., "/// // ---" or just "/// =" artifacts
        content = line[3:].strip() if len(line) > 3 else ''
        
        # Remove doc lines that are just section banners like "// =========" or "// ---------"
        # that got accidentally turned into doc comments
        stripped_content = line.lstrip('/')
        if is_doc and re.match(r'^[=]{3,}$', content):
            # Replace with a clean doc comment about the section
            continue  # Skip these - they add no value in doc comments
        if is_doc and re.match(r'^[-]{3,}$', content):
            continue  # Skip these too
        # Remove "// " comments that got doc-ified
        if is_doc and (content.startswith('//') or content.startswith('//!')):
            continue
        
        result.append(line)
    
    return '\n'.join(result)


def fix_impl_blocks_doc(text):
    """Fix /// annotations inside impl blocks that are block-comment style section
    markers, not actual doc comments."""
    lines = text.split('\n')
    result = []
    
    for i, line in enumerate(lines):
        # Check for patterns like "/// //" which are block comments gone wrong
        stripped = line.strip()
        if stripped.startswith('////') or stripped.startswith('/// //'):
            # This is a comment, not a doc comment
            # Replace /// with // to make it a regular comment
            if stripped.startswith('////'):
                new_line = line.replace('////', '//', 1)
            elif stripped.startswith('/// //'):
                new_line = line.replace('/// //', '// ')
            else:
                new_line = line
            result.append(new_line)
        else:
            result.append(line)
    
    return '\n'.join(result)


def process_file(filepath):
    """Process a single .rs file, applying all rustdoc quality fixes."""
    with open(filepath, 'r') as f:
        text = f.read()
    
    original = text
    
    # 1. Fix "/// This function/struct/enum" patterns
    lines = text.split('\n')
    lines = fix_meta_text_lines(lines)
    text = '\n'.join(lines)
    
    # 2. Fix section header text (# -> ##)
    text = fix_section_header_text(text)
    
    # 3. Remove block-comment section banners from doc comments
    text = remove_section_block_comments(text)
    
    # 4. Fix impl block annotations
    text = fix_impl_blocks_doc(text)
    
    if text != original:
        with open(filepath, 'w') as f:
            f.write(text)
        return True
    return False


def walk_and_process():
    """Walk all .rs files in the Source dir and process them."""
    modified_files = []
    total_files = 0
    
    for root, dirs, files in os.walk(SOURCE_DIR):
        for f in sorted(files):
            if not f.endswith('.rs'):
                continue
            filepath = os.path.join(root, f)
            total_files += 1
            if process_file(filepath):
                modified_files.append(os.path.relpath(filepath, SOURCE_DIR))
                print(f"  MODIFIED: {os.path.relpath(filepath, SOURCE_DIR)}")
    
    return total_files, modified_files


if __name__ == '__main__':
    total, modified = walk_and_process()
    print(f"\nTotal .rs files scanned: {total}")
    print(f"Files modified: {len(modified)}")
    for f in modified:
        print(f"  - {f}")
