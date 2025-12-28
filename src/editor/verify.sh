#!/bin/bash

# Diagram Editor Module Verification Script
# Run this to verify the module is complete and ready

echo "=================================="
echo "DIAGRAM EDITOR MODULE VERIFICATION"
echo "=================================="
echo ""

# Check all required files exist
echo "Checking required files..."
FILES=(
    "index.ts"
    "DiagramCanvas.tsx"
    "DiagramToolbar.tsx"
    "DiagramState.ts"
    "DiagramElements.ts"
    "DiagramExporter.ts"
    "VehiclePath.ts"
    "MeasurementTool.ts"
    "../types/diagram.ts"
)

MISSING=0
for file in "${FILES[@]}"; do
    if [ -f "/home/user/accident-recreate/src/editor/$file" ]; then
        echo "✓ $file"
    else
        echo "✗ $file - MISSING"
        MISSING=$((MISSING + 1))
    fi
done

echo ""
echo "Documentation files..."
DOCS=(
    "README.md"
    "INTEGRATION.md"
    "COMPLETION_SUMMARY.md"
    "DiagramEditorExample.tsx"
)

for file in "${DOCS[@]}"; do
    if [ -f "/home/user/accident-recreate/src/editor/$file" ]; then
        echo "✓ $file"
    else
        echo "✗ $file - MISSING"
        MISSING=$((MISSING + 1))
    fi
done

echo ""
echo "Test files..."
if [ -f "/home/user/accident-recreate/src/editor/__tests__/DiagramEditor.test.ts" ]; then
    echo "✓ DiagramEditor.test.ts"
else
    echo "✗ DiagramEditor.test.ts - MISSING"
    MISSING=$((MISSING + 1))
fi

echo ""
echo "=================================="
echo "Code Statistics"
echo "=================================="

# Count lines of code
TOTAL_LINES=$(wc -l /home/user/accident-recreate/src/editor/*.ts /home/user/accident-recreate/src/editor/*.tsx /home/user/accident-recreate/src/types/diagram.ts 2>/dev/null | tail -1 | awk '{print $1}')
echo "Total Lines of Code: $TOTAL_LINES"

# Count files
FILE_COUNT=$(find /home/user/accident-recreate/src/editor -type f | wc -l)
echo "Total Files: $FILE_COUNT"

# Show file sizes
echo ""
echo "File Sizes:"
ls -lh /home/user/accident-recreate/src/editor/*.ts /home/user/accident-recreate/src/editor/*.tsx 2>/dev/null | awk '{print $5 "\t" $9}' | sed 's|.*/||'

echo ""
echo "=================================="
echo "Verification Result"
echo "=================================="

if [ $MISSING -eq 0 ]; then
    echo "✅ ALL CHECKS PASSED"
    echo "Module is COMPLETE and ready for integration"
    exit 0
else
    echo "❌ $MISSING FILES MISSING"
    echo "Module is INCOMPLETE"
    exit 1
fi
