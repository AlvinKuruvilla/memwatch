#!/usr/bin/env python3
"""
Packaging Script for Build Pipeline Example

Creates distribution packages with configurable memory usage.
Simulates realistic packaging operations with file copying, compression, etc.

Usage:
    python3 package.py --size small --binary target/debug/build_example --output dist
"""

import argparse
import os
import sys
import shutil
import tarfile
import json
import hashlib
from pathlib import Path
from datetime import datetime


# Problem size configurations
SIZES = {
    'small': {
        'dummy_files': 10,
        'file_size_kb': 100,
        'description': 'Quick packaging (~10 MB)'
    },
    'medium': {
        'dummy_files': 50,
        'file_size_kb': 500,
        'description': 'Moderate packaging (~50 MB)'
    },
    'large': {
        'dummy_files': 100,
        'file_size_kb': 1000,
        'description': 'Large packaging (~200 MB)'
    }
}


def create_dummy_files(output_dir, count, size_kb):
    """Create dummy files to simulate package contents."""

    dummy_dir = output_dir / 'dummy_data'
    dummy_dir.mkdir(parents=True, exist_ok=True)

    print(f"  Creating {count} dummy files ({size_kb} KB each)...")

    files = []
    for i in range(count):
        file_path = dummy_dir / f'data_{i:04d}.bin'

        # Generate pseudo-random data
        data = bytearray()
        for j in range(size_kb):
            # Simple pseudo-random byte generation
            byte_val = (i * 256 + j * 17) % 256
            data.extend([byte_val] * 1024)

        with open(file_path, 'wb') as f:
            f.write(data)

        files.append(file_path)

    return files


def compute_checksums(files):
    """Compute SHA256 checksums for all files (memory intensive)."""

    print(f"  Computing checksums for {len(files)} files...")

    checksums = {}
    for file_path in files:
        hasher = hashlib.sha256()

        # Read file in chunks (simulates I/O + memory usage)
        with open(file_path, 'rb') as f:
            while chunk := f.read(8192):
                hasher.update(chunk)

        checksums[str(file_path)] = hasher.hexdigest()

    return checksums


def create_metadata(binary_path, checksums, size):
    """Create package metadata."""

    print("  Creating package metadata...")

    metadata = {
        'package': 'build-example',
        'version': '1.0.0',
        'created': datetime.now().isoformat(),
        'size_config': size,
        'binary': str(binary_path),
        'file_count': len(checksums),
        'checksums': checksums
    }

    return metadata


def create_tarball(output_dir, files, binary_path, metadata):
    """Create compressed tarball (memory intensive)."""

    print("  Creating compressed tarball...")

    tarball_path = output_dir / 'package.tar.gz'

    with tarfile.open(tarball_path, 'w:gz') as tar:
        # Add binary
        if os.path.exists(binary_path):
            tar.add(binary_path, arcname=f"bin/{os.path.basename(binary_path)}")

        # Add dummy files
        for file_path in files:
            tar.add(file_path, arcname=f"data/{os.path.basename(file_path)}")

        # Add metadata
        metadata_path = output_dir / 'metadata.json'
        with open(metadata_path, 'w') as f:
            json.dump(metadata, f, indent=2)
        tar.add(metadata_path, arcname='metadata.json')

    return tarball_path


def simulate_file_processing(files):
    """Simulate additional file processing (memory operations)."""

    print("  Simulating file processing...")

    # Read all files into memory (simulates batch processing)
    file_contents = []
    for file_path in files[:min(10, len(files))]:  # Limit to avoid excessive memory
        with open(file_path, 'rb') as f:
            data = f.read()
            file_contents.append(data)

    # Process data (simple transformation)
    processed = []
    for data in file_contents:
        # Simulate transformation (creates new buffer)
        transformed = bytes([b ^ 0xFF for b in data[:1024]])  # XOR first KB
        processed.append(transformed)

    total_processed = sum(len(p) for p in processed)
    print(f"  Processed {total_processed} bytes")

    return total_processed > 0


def main():
    parser = argparse.ArgumentParser(
        description='Create distribution package for build pipeline example'
    )
    parser.add_argument(
        '--size',
        choices=['small', 'medium', 'large'],
        default='small',
        help='Problem size (default: small)'
    )
    parser.add_argument(
        '--binary',
        type=Path,
        required=True,
        help='Path to compiled binary'
    )
    parser.add_argument(
        '--output',
        type=Path,
        required=True,
        help='Output directory for package'
    )

    args = parser.parse_args()

    # Get configuration
    config = SIZES[args.size]

    print("=" * 50)
    print("Packaging for Build Pipeline Example")
    print("=" * 50)
    print(f"Size: {args.size}")
    print(f"Description: {config['description']}")
    print(f"Dummy files: {config['dummy_files']}")
    print(f"File size: {config['file_size_kb']} KB")
    print()

    # Create output directory
    output_dir = args.output
    output_dir.mkdir(parents=True, exist_ok=True)

    # Check binary exists
    if not args.binary.exists():
        print(f"Warning: Binary not found at {args.binary}")
        print("Continuing without binary...")

    # Step 1: Create dummy files
    print("Step 1: Creating dummy files")
    files = create_dummy_files(output_dir, config['dummy_files'], config['file_size_kb'])
    print(f"  ✓ Created {len(files)} files")
    print()

    # Step 2: Compute checksums
    print("Step 2: Computing checksums")
    checksums = compute_checksums(files)
    print(f"  ✓ Computed {len(checksums)} checksums")
    print()

    # Step 3: Process files
    print("Step 3: Processing files")
    if simulate_file_processing(files):
        print("  ✓ File processing complete")
    print()

    # Step 4: Create metadata
    print("Step 4: Creating metadata")
    metadata = create_metadata(args.binary, checksums, args.size)
    print(f"  ✓ Metadata created")
    print()

    # Step 5: Create tarball
    print("Step 5: Creating tarball")
    tarball_path = create_tarball(output_dir, files, args.binary, metadata)
    tarball_size_mb = tarball_path.stat().st_size / (1024 * 1024)
    print(f"  ✓ Tarball created: {tarball_path.name} ({tarball_size_mb:.1f} MB)")
    print()

    # Calculate total package size
    total_size_mb = sum(f.stat().st_size for f in files) / (1024 * 1024)

    print("=" * 50)
    print("Packaging Complete")
    print("=" * 50)
    print(f"Files: {len(files)}")
    print(f"Total size: {total_size_mb:.1f} MB")
    print(f"Tarball: {tarball_size_mb:.1f} MB")
    print(f"Output: {output_dir}")
    print()


if __name__ == '__main__':
    try:
        main()
    except KeyboardInterrupt:
        print("\n\nInterrupted by user")
        sys.exit(1)
    except Exception as e:
        print(f"\nError: {e}", file=sys.stderr)
        import traceback
        traceback.print_exc()
        sys.exit(1)
