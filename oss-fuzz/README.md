# OSS-Fuzz Integration

This directory contains the configuration files for integrating Vexy JSON with OSS-Fuzz, Google's continuous fuzzing service for open source projects.

## Files

- `project.yaml` - Main project configuration
- `build.sh` - Build script for OSS-Fuzz
- `Dockerfile` - Container configuration
- `README.md` - This file

## Setup

To set up OSS-Fuzz integration:

1. Fork the [OSS-Fuzz repository](https://github.com/google/oss-fuzz)
2. Create a new directory under `projects/vexy-json/`
3. Copy the files from this directory to `projects/vexy-json/`
4. Submit a pull request to the OSS-Fuzz repository

## Testing Locally

To test the OSS-Fuzz integration locally:

```bash
# Clone OSS-Fuzz
git clone https://github.com/google/oss-fuzz.git
cd oss-fuzz

# Copy project files
cp -r /path/to/vexy_json/oss-fuzz projects/vexy-json/

# Build the project
python infra/helper.py build_image vexy_json
python infra/helper.py build_fuzzers vexy_json

# Run fuzzers
python infra/helper.py run_fuzzer vexy_json json_structure
```

## Fuzzing Targets

The following fuzz targets are included:

- `json_structure` - Tests overall JSON structure parsing
- `json_strings` - Tests string parsing and escaping
- `unquoted_keys` - Tests unquoted key parsing
- `unicode` - Tests Unicode handling
- `repair` - Tests repair functionality
- `streaming` - Tests streaming parser

## Coverage

Coverage reports are automatically generated and can be viewed at:
https://storage.googleapis.com/oss-fuzz-coverage/vexy_json/latest/index.html

## Bug Reports

When OSS-Fuzz finds bugs, they are automatically reported to the GitHub issue tracker with the label `oss-fuzz`.

## Corpus

The fuzzing corpus is continuously grown and improved. Initial seed inputs are provided from:

- Real-world JSON files
- Edge cases and corner cases
- Previously discovered bug-triggering inputs

## Configuration

The fuzzing configuration includes:

- Multiple fuzzing engines (libfuzzer, AFL, honggfuzz)
- Multiple sanitizers (AddressSanitizer, UndefinedBehaviorSanitizer, MemorySanitizer)
- Custom JSON dictionary for better input generation
- Comprehensive corpus seeding

## Maintenance

The OSS-Fuzz integration requires minimal maintenance:

- Build script updates when dependencies change
- Corpus updates when new edge cases are discovered
- Configuration updates when new fuzz targets are added