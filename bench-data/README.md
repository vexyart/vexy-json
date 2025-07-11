# Benchmark Data Files

This directory contains real-world JSON files used for comprehensive benchmarking of the Vexy JSON parser.

## File Categories

### Small Files (1-10KB)
- Configuration files
- API responses
- Package manifests

### Medium Files (10-100KB)
- API responses with multiple records
- GeoJSON features
- Database dumps

### Large Files (100MB-1GB)
- Complete API datasets
- Log files
- Large GeoJSON collections

## Usage

These files are used by the benchmark suite to test:
- Parsing performance across different file sizes
- Memory usage patterns
- Real-world compatibility
- Edge case handling

## Data Sources

Files are collected from:
- Public APIs (Twitter, GitHub, etc.)
- Open datasets
- Generated test data
- Community contributions

## Adding New Files

To add new benchmark data:

1. Place files in the appropriate size category subdirectory
2. Update the benchmark suite to include the new files
3. Document the source and characteristics of the data
4. Ensure no sensitive information is included

## File Naming Convention

- `config_*.json` - Configuration files
- `api_*.json` - API responses
- `geo_*.json` - GeoJSON data
- `logs_*.json` - Log files in JSON format
- `generated_*.json` - Synthetically generated data