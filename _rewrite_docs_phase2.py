#!/usr/bin/env python3
"""
Phase 2: Deep rustdoc quality rewrite for the Maintain element crate.

Fixes that require structural understanding of the file:
1. Fix empty /// lines that are between code comments (not adjacent to doc comments)
2. Ensure all non-code-wrapper // comments are not /// doc comments
3. Add #[allow(missing_docs)] for private items that can't reasonably be documented
4. Clean up redundant /// lines around section headers
"""

import re
import os

SOURCE_DIR = "/Volumes/CORSAIR/Developer/macOS/Application/CodeEditorLand/Land/Element/Maintain/Source"


def classify_lines(text):
    """
    Classify each line and fix issues:
    - Lines that are // ... comments formatted as /// ... (should be // not ///)
    - Empty /// lines that are between // comment blocks
    """
    lines = text.split('\n')
    result = []
    
    for i, line in enumerate(lines):
        stripped = line
        
        # Fix "//=" and "//-" section banners that got prefixed with extra /
        # e.g., "////" or "/////" section banners
        # Pattern: starts with 4+ slashes
        m = re.match(r'^(/+)([/!=-].*)', stripped)
        if m:
            slashes = m.group(1)
            rest = m.group(2)
            if len(slashes) >= 4 and rest and rest[0] in ('/', '!', '=', '-'):
                # This is a comment section marker with too many slashes
                # Change ///// === --> // ===  (keep exactly 2 slashes)
                if rest.startswith('/'):
                    # //// some comment --> // some comment
                    new_line = '//' + rest[1:]
                elif rest.startswith('!'):
                    # ////! doc comment -> //! doc comment  
                    new_line = '//!' + rest[1:]
                else:
                    new_line = '//' + rest
                result.append(new_line)
                continue
        
        # Fix "/// " lines that are actually // comments (architecture section banners)
        # Patterns like "/// // =====" or "/// // --------"
        if re.match(r'^///\s*//', stripped):
            # This is a // comment inside a doc comment - change to regular comment
            result.append('//' + stripped[3:])
            continue
        
        result.append(line)
    
    return '\n'.join(result)


def process_file(filepath):
    with open(filepath, 'r') as f:
        text = f.read()
    
    original = text
    
    # Fix comments
    text = classify_lines(text)
    
    if text != original:
        with open(filepath, 'w') as f:
            f.write(text)
        return True
    return False


def walk_and_process():
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
