#!/usr/bin/env python3
"""Example: Using vexy_json for configuration files"""

import vexy_json
import sys

# Example configuration with forgiving JSON features
CONFIG_TEMPLATE = """
{
    // Application configuration
    app: {
        name: 'MyApp',
        version: '1.0.0',
        debug: true,  // Enable debug mode in development
    },
    
    // Server settings
    server: {
        host: 'localhost',
        port: 8080,
        workers: 4
        
        // SSL configuration (optional)
        ssl: {
            enabled: false,
            cert: '/path/to/cert.pem',
            key: '/path/to/key.pem',
        }
    },
    
    // Database configuration
    database: {
        engine: 'postgresql',
        host: 'localhost',
        port: 5432,
        name: 'myapp_db',
        user: 'myapp_user',
        /* Password should be in environment variable */
        password_env: 'DB_PASSWORD',
    },
    
    // Feature flags
    features: {
        new_ui: true
        analytics: false
        beta_features: [
            'dark_mode',
            'export_pdf',
            'advanced_search',
        ]
    },
    
    // Logging configuration
    logging: {
        level: 'INFO',  // DEBUG, INFO, WARNING, ERROR
        format: '[%(asctime)s] %(levelname)s: %(message)s',
        handlers: [
            {
                type: 'console',
                level: 'DEBUG',
            },
            {
                type: 'file',
                filename: '/var/log/myapp.log',
                level: 'INFO',
                max_size: '10MB',
                backup_count: 5,
            }
        ],
    },
}
"""


def load_config(filename=None):
    """Load configuration from file or use default template"""
    if filename:
        try:
            config = vexy_json.load(filename)
            print(f"Loaded configuration from: {filename}")
        except Exception as e:
            print(f"Error loading config file: {e}")
            sys.exit(1)
    else:
        print("Using default configuration template")
        config = vexy_json.parse(CONFIG_TEMPLATE)

    return config


def print_config(config, indent=0):
    """Pretty print configuration"""
    prefix = "  " * indent

    if isinstance(config, dict):
        for key, value in config.items():
            if isinstance(value, (dict, list)):
                print(f"{prefix}{key}:")
                print_config(value, indent + 1)
            else:
                print(f"{prefix}{key}: {value}")
    elif isinstance(config, list):
        for item in config:
            if isinstance(item, (dict, list)):
                print(f"{prefix}-")
                print_config(item, indent + 1)
            else:
                print(f"{prefix}- {item}")


def validate_config(config):
    """Validate configuration structure"""
    required_sections = ["app", "server", "database"]

    for section in required_sections:
        if section not in config:
            raise ValueError(f"Missing required section: {section}")

    # Validate server settings
    if not isinstance(config["server"].get("port"), int):
        raise ValueError("Server port must be an integer")

    if not 1 <= config["server"]["port"] <= 65535:
        raise ValueError("Server port must be between 1 and 65535")

    # Validate database settings
    valid_engines = ["postgresql", "mysql", "sqlite"]
    if config["database"].get("engine") not in valid_engines:
        raise ValueError(f"Database engine must be one of: {valid_engines}")

    print("Configuration validation passed!")


def main():
    print("vexy_json Configuration Parser Example")
    print("=" * 50)

    # Load configuration
    config_file = sys.argv[1] if len(sys.argv) > 1 else None
    config = load_config(config_file)

    # Display configuration
    print("\nLoaded Configuration:")
    print("-" * 30)
    print_config(config)

    # Validate configuration
    print("\nValidating configuration...")
    try:
        validate_config(config)
    except ValueError as e:
        print(f"Validation error: {e}")
        sys.exit(1)

    # Example: Access specific values
    print("\nAccessing configuration values:")
    print(f"App name: {config['app']['name']}")
    print(f"Debug mode: {config['app']['debug']}")
    print(f"Server: {config['server']['host']}:{config['server']['port']}")
    print(f"Database: {config['database']['engine']} @ {config['database']['host']}")

    # Example: Save configuration
    if not config_file:
        output_file = "config_example.json"
        vexy_json.dump(config, output_file, indent=2)
        print(f"\nSaved example configuration to: {output_file}")


if __name__ == "__main__":
    main()
