// this_file: tests/real_world_scenarios.rs

//! Real-world scenario tests for forgiving JSON compatibility
//!
//! This module tests realistic use cases and scenarios that users might encounter
//! when using forgiving JSON for configuration files, data exchange, and other practical applications.

use rustc_hash::FxHashMap;
use vexy_json::{parse, Value};
use vexy_json_core::{parse_with_options, ParserOptions};

/// Helper functions for creating test values
#[allow(dead_code)]
fn obj(pairs: &[(&str, Value)]) -> Value {
    let mut map = FxHashMap::default();
    for (k, v) in pairs {
        map.insert(k.to_string(), v.clone());
    }
    Value::Object(map)
}

#[allow(dead_code)]
fn arr(values: Vec<Value>) -> Value {
    Value::Array(values)
}

fn s(text: &str) -> Value {
    Value::String(text.to_string())
}

fn n(num: i64) -> Value {
    Value::Number(vexy_json::Number::Integer(num))
}

#[allow(dead_code)]
fn f(num: f64) -> Value {
    Value::Number(vexy_json::Number::Float(num))
}

fn b(val: bool) -> Value {
    Value::Bool(val)
}

fn null() -> Value {
    Value::Null
}

/// Tests based on real configuration file scenarios
mod configuration_files {
    use super::*;

    #[test]
    fn test_web_server_config() {
        let input = r#"
        {
            // Server configuration
            server: {
                host: "0.0.0.0",
                port: 8080,
                ssl: {
                    enabled: true,
                    cert_path: "/etc/ssl/certs/server.crt",
                    key_path: "/etc/ssl/private/server.key"
                }
            },
            
            // Database configuration
            database: {
                type: "postgresql",
                host: "localhost",
                port: 5432,
                database: "myapp",
                connection_pool: {
                    min_connections: 5,
                    max_connections: 20,
                    timeout: 30000
                }
            },
            
            // Logging configuration
            logging: {
                level: "info",
                format: "json",
                outputs: ["console", "file"],
                file_config: {
                    path: "/var/log/app.log",
                    max_size: "100MB",
                    rotate: true
                }
            },
            
            // Feature flags
            features: {
                authentication: true,
                rate_limiting: true,
                caching: false,
                debug_mode: false
            }
        }
        "#;

        let result = parse(input).unwrap();

        // Verify server config
        assert_eq!(result["server"]["host"], s("0.0.0.0"));
        assert_eq!(result["server"]["port"], n(8080));
        assert_eq!(result["server"]["ssl"]["enabled"], b(true));

        // Verify database config
        assert_eq!(result["database"]["type"], s("postgresql"));
        assert_eq!(
            result["database"]["connection_pool"]["max_connections"],
            n(20)
        );

        // Verify logging config
        if let Value::Array(outputs) = &result["logging"]["outputs"] {
            assert_eq!(outputs.len(), 2);
            assert_eq!(outputs[0], s("console"));
            assert_eq!(outputs[1], s("file"));
        }

        // Verify feature flags
        assert_eq!(result["features"]["authentication"], b(true));
        assert_eq!(result["features"]["debug_mode"], b(false));
    }

    #[test]
    fn test_build_configuration() {
        let input = r#"
        {
            // Build system configuration
            name: "my-awesome-app",
            version: "1.2.3",
            
            scripts: {
                build: "webpack --mode production",
                dev: "webpack serve --mode development",
                test: "jest",
                lint: "eslint src/"
            },
            
            dependencies: [
                "react@^18.0.0",
                "lodash@^4.17.21",
                "axios@^1.0.0"
            ],
            
            build: {
                target: "es2020",
                outDir: "dist/",
                sourceMaps: true,
                minify: true,
                bundle_analysis: false
            },
            
            environments: {
                development: {
                    api_url: "http://localhost:3000",
                    debug: true
                },
                staging: {
                    api_url: "https://staging-api.example.com",
                    debug: false
                },
                production: {
                    api_url: "https://api.example.com",
                    debug: false
                }
            }
        }
        "#;

        let result = parse(input).unwrap();

        assert_eq!(result["name"], s("my-awesome-app"));
        assert_eq!(result["version"], s("1.2.3"));
        assert_eq!(result["scripts"]["build"], s("webpack --mode production"));

        if let Value::Array(deps) = &result["dependencies"] {
            assert_eq!(deps.len(), 3);
            assert!(deps
                .iter()
                .any(|d| matches!(d, Value::String(s) if s.starts_with("react@"))));
        }

        assert_eq!(
            result["environments"]["production"]["api_url"],
            s("https://api.example.com")
        );
    }

    #[test]
    fn test_docker_compose_style() {
        let input = r#"
        {
            version: "3.8",
            
            services: {
                web: {
                    image: "nginx:alpine",
                    ports: ["80:80", "443:443"],
                    volumes: [
                        "./nginx.conf:/etc/nginx/nginx.conf",
                        "./ssl:/etc/ssl"
                    ],
                    environment: {
                        NGINX_HOST: "example.com",
                        NGINX_PORT: 80
                    },
                    depends_on: ["api"]
                },
                
                api: {
                    build: {
                        context: "./api",
                        dockerfile: "Dockerfile"
                    },
                    ports: ["3000:3000"],
                    environment: {
                        NODE_ENV: "production",
                        DATABASE_URL: "postgresql://user:pass@db:5432/myapp"
                    },
                    depends_on: ["db"]
                },
                
                db: {
                    image: "postgres:13",
                    environment: {
                        POSTGRES_DB: "myapp",
                        POSTGRES_USER: "user",
                        POSTGRES_PASSWORD: "pass"
                    },
                    volumes: ["postgres_data:/var/lib/postgresql/data"]
                }
            },
            
            volumes: {
                postgres_data: null
            }
        }
        "#;

        let result = parse(input).unwrap();

        assert_eq!(result["version"], s("3.8"));
        assert_eq!(result["services"]["web"]["image"], s("nginx:alpine"));
        assert_eq!(
            result["services"]["api"]["environment"]["NODE_ENV"],
            s("production")
        );
        assert_eq!(result["volumes"]["postgres_data"], null());
    }
}

/// Tests for data interchange scenarios
mod data_interchange {
    use super::*;

    #[test]
    fn test_api_response_style() {
        let input = r#"
        {
            status: "success",
            code: 200,
            message: "Data retrieved successfully",
            
            data: {
                users: [
                    {
                        id: 1,
                        username: "alice",
                        email: "alice@example.com",
                        profile: {
                            first_name: "Alice",
                            last_name: "Johnson",
                            age: 28,
                            preferences: {
                                theme: "dark",
                                notifications: true,
                                language: "en"
                            }
                        },
                        roles: ["user", "moderator"],
                        created_at: "2023-01-15T10:30:00Z",
                        last_login: "2024-01-07T14:22:33Z"
                    },
                    {
                        id: 2,
                        username: "bob",
                        email: "bob@example.com",
                        profile: {
                            first_name: "Bob",
                            last_name: "Smith",
                            age: 35,
                            preferences: {
                                theme: "light",
                                notifications: false,
                                language: "es"
                            }
                        },
                        roles: ["user"],
                        created_at: "2023-03-22T09:15:00Z",
                        last_login: "2024-01-06T16:45:12Z"
                    }
                ],
                
                pagination: {
                    page: 1,
                    per_page: 20,
                    total: 2,
                    total_pages: 1
                }
            },
            
            meta: {
                request_id: "req_12345",
                timestamp: "2024-01-07T15:30:00Z",
                server: "api-server-01"
            }
        }
        "#;

        let result = parse(input).unwrap();

        assert_eq!(result["status"], s("success"));
        assert_eq!(result["code"], n(200));

        if let Value::Array(users) = &result["data"]["users"] {
            assert_eq!(users.len(), 2);
            assert_eq!(users[0]["username"], s("alice"));
            assert_eq!(users[0]["profile"]["age"], n(28));
            assert_eq!(users[1]["username"], s("bob"));

            if let Value::Array(roles) = &users[0]["roles"] {
                assert_eq!(roles.len(), 2);
                assert_eq!(roles[0], s("user"));
                assert_eq!(roles[1], s("moderator"));
            }
        }

        assert_eq!(result["data"]["pagination"]["total"], n(2));
        assert_eq!(result["meta"]["request_id"], s("req_12345"));
    }

    #[test]
    fn test_geojson_style() {
        let input = r#"
        {
            type: "FeatureCollection",
            features: [
                {
                    type: "Feature",
                    geometry: {
                        type: "Point",
                        coordinates: [-122.4194, 37.7749] // San Francisco
                    },
                    properties: {
                        name: "San Francisco",
                        population: 881549,
                        state: "California",
                        country: "USA"
                    }
                },
                {
                    type: "Feature",
                    geometry: {
                        type: "Polygon",
                        coordinates: [[
                            [-122.366, 37.816],
                            [-122.365, 37.816],
                            [-122.365, 37.815],
                            [-122.366, 37.815],
                            [-122.366, 37.816]
                        ]]
                    },
                    properties: {
                        name: "Sample Area",
                        area_type: "park"
                    }
                }
            ]
        }
        "#;

        let result = parse(input).unwrap();

        assert_eq!(result["type"], s("FeatureCollection"));

        if let Value::Array(features) = &result["features"] {
            assert_eq!(features.len(), 2);

            // Check point feature
            assert_eq!(features[0]["type"], s("Feature"));
            assert_eq!(features[0]["geometry"]["type"], s("Point"));
            if let Value::Array(coords) = &features[0]["geometry"]["coordinates"] {
                assert_eq!(coords.len(), 2);
            }
            assert_eq!(features[0]["properties"]["name"], s("San Francisco"));

            // Check polygon feature
            assert_eq!(features[1]["geometry"]["type"], s("Polygon"));
            assert_eq!(features[1]["properties"]["area_type"], s("park"));
        }
    }

    #[test]
    fn test_log_entry_format() {
        let input = r#"
        {
            timestamp: "2024-01-07T15:30:45.123Z",
            level: "error",
            service: "user-service",
            version: "v2.1.3",
            
            message: "Failed to authenticate user",
            
            context: {
                user_id: "user_12345",
                session_id: "sess_abcdef",
                ip_address: "192.168.1.100",
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
            },
            
            error: {
                type: "AuthenticationError",
                code: "AUTH_001",
                message: "Invalid credentials provided",
                stack_trace: [
                    "at authenticate (/app/auth.js:42:15)",
                    "at loginHandler (/app/routes/auth.js:23:8)",
                    "at Layer.handle [as handle_request] (/app/node_modules/express/lib/router/layer.js:95:5)"
                ]
            },
            
            request: {
                method: "POST",
                url: "/api/auth/login",
                headers: {
                    "content-type": "application/json",
                    "user-agent": "curl/7.68.0"
                },
                body_size: 156
            },
            
            response: {
                status_code: 401,
                duration_ms: 23.5,
                body_size: 89
            },
            
            tags: ["authentication", "security", "failure"]
        }
        "#;

        let result = parse(input).unwrap();

        assert_eq!(result["level"], s("error"));
        assert_eq!(result["service"], s("user-service"));
        assert_eq!(result["message"], s("Failed to authenticate user"));
        assert_eq!(result["context"]["user_id"], s("user_12345"));
        assert_eq!(result["error"]["type"], s("AuthenticationError"));
        assert_eq!(result["request"]["method"], s("POST"));
        assert_eq!(result["response"]["status_code"], n(401));

        if let Value::Array(tags) = &result["tags"] {
            assert!(tags.contains(&s("authentication")));
            assert!(tags.contains(&s("security")));
        }
    }
}

/// Tests for configuration migration scenarios
mod migration_scenarios {
    use super::*;

    #[test]
    fn test_json_to_vexy_json_migration() {
        // Start with strict JSON
        let strict_json = r#"{
            "database": {
                "host": "localhost",
                "port": 5432,
                "ssl": true
            },
            "cache": {
                "type": "redis",
                "ttl": 3600
            }
        }"#;

        // Migrate with comments and unquoted keys
        let vexy_json_version = r#"{
            // Database configuration
            database: {
                host: "localhost",
                port: 5432,
                ssl: true
            },
            
            // Cache configuration  
            cache: {
                type: "redis",
                ttl: 3600 // 1 hour
            }
        }"#;

        let json_result = parse(strict_json).unwrap();
        let jsonic_result = parse(vexy_json_version).unwrap();

        // Both should parse to equivalent structures
        assert_eq!(
            json_result["database"]["host"],
            jsonic_result["database"]["host"]
        );
        assert_eq!(
            json_result["database"]["port"],
            jsonic_result["database"]["port"]
        );
        assert_eq!(json_result["cache"]["type"], jsonic_result["cache"]["type"]);
        assert_eq!(json_result["cache"]["ttl"], jsonic_result["cache"]["ttl"]);
    }

    #[test]
    fn test_yaml_like_syntax() {
        // YAML-inspired syntax
        let input = r#"{
        name: "my-app"
        version: "1.0.0"
        
        dependencies: [
          { name: "lodash", version: "4.17.21" },
          { name: "axios", version: "1.0.0" }
        ]
            
        config: {
          database: {
            host: "localhost"
            port: 5432
          }
          
          redis: {
            host: "localhost"
            port: 6379
          }
        }
        }"#;

        let options = ParserOptions {
            allow_unquoted_keys: true,
            implicit_top_level: true,
            newline_as_comma: true,
            ..Default::default()
        };
        let result = parse_with_options(input, options).unwrap();

        assert_eq!(result["name"], s("my-app"));
        assert_eq!(result["version"], s("1.0.0"));

        if let Value::Array(deps) = &result["dependencies"] {
            assert_eq!(deps.len(), 2);
            assert_eq!(deps[0]["name"], s("lodash"));
            assert_eq!(deps[1]["name"], s("axios"));
        }

        assert_eq!(result["config"]["database"]["host"], s("localhost"));
        assert_eq!(result["config"]["redis"]["port"], n(6379));
    }
}

/// Tests for error recovery and resilience
mod error_recovery {
    use super::*;

    #[test]
    fn test_partial_parsing_resilience() {
        // Test that valid parts can be extracted even with some issues
        let inputs = vec![
            r#"{valid: "data", /* unclosed comment "#, // Should error gracefully
            r#"{"valid": "data", "invalid": }"#,       // Should error on invalid value
            r#"[1, 2, 3, {"broken": }]"#,              // Should error on object
        ];

        for input in inputs {
            let result = parse(input);
            // Should fail but not panic
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_large_input_handling() {
        // Generate a large but valid input
        let mut input = String::from("{\n");

        for i in 0..1000 {
            input.push_str(&format!("  key{i}: \"value{i}\",\n"));
        }

        input.push_str("  final_key: \"final_value\"\n}");

        let result = parse(&input);
        assert!(result.is_ok(), "Large input should parse successfully");

        if let Ok(Value::Object(obj)) = result {
            assert_eq!(obj.len(), 1001); // 1000 + final_key
            assert_eq!(obj["key0"], s("value0"));
            assert_eq!(obj["key999"], s("value999"));
            assert_eq!(obj["final_key"], s("final_value"));
        }
    }

    #[test]
    fn test_deeply_nested_input() {
        // Test parser's handling of deep nesting
        let mut input = String::new();
        let depth = 50;

        // Create nested objects: {a: {a: {a: ...}}}
        for _ in 0..depth {
            input.push_str("{a: ");
        }
        input.push_str("\"deep_value\"");
        for _ in 0..depth {
            input.push('}');
        }

        let result = parse(&input);
        // Should either succeed or fail gracefully with depth limit
        match result {
            Ok(parsed) => {
                // If it succeeds, verify we can access the deep value
                let mut current = &parsed;
                for _ in 0..depth {
                    current = &current["a"];
                }
                assert_eq!(*current, s("deep_value"));
            }
            Err(_) => {
                // Depth limit exceeded - this is acceptable behavior
            }
        }
    }
}

/// Performance and benchmarking scenarios
mod performance_scenarios {
    use super::*;

    #[test]
    fn test_repeated_structures() {
        // Test parsing of repeated similar structures (like log entries)
        let mut input = String::from("[\n");

        for i in 0..100 {
            if i > 0 {
                input.push(',');
            }
            input.push_str(&format!(
                r#"
            {{
                id: {},
                timestamp: "2024-01-07T15:30:{:02}.000Z",
                level: "info",
                message: "Processing request {}",
                context: {{
                    user_id: "user_{}",
                    action: "view_page",
                    page: "/dashboard"
                }}
            }}"#,
                i,
                i % 60,
                i,
                i
            ));
        }

        input.push_str("\n]");

        let start = std::time::Instant::now();
        let result = parse(&input);
        let duration = start.elapsed();

        assert!(
            result.is_ok(),
            "Repeated structures should parse successfully"
        );
        assert!(
            duration.as_millis() < 1000,
            "Parsing should be reasonably fast"
        );

        if let Ok(Value::Array(arr)) = result {
            assert_eq!(arr.len(), 100);
            assert_eq!(arr[0]["id"], n(0));
            assert_eq!(arr[99]["id"], n(99));
        }
    }

    #[test]
    fn test_varied_data_types() {
        // Test parsing of mixed data types in a single structure
        let input = r#"
        {
            strings: ["short", "medium length string", "very long string that contains multiple words and spans quite a bit of text"],
            numbers: [0, 1, -1, 1000000, -1000000, 3.14159, -2.71828, 1e10, 1e-10],
            booleans: [true, false, true, false],
            nulls: [null, null, null],
            objects: [
                {name: "object1", value: 1},
                {name: "object2", value: 2, nested: {deep: true}},
                {}
            ],
            arrays: [
                [],
                [1],
                [1, 2, 3],
                [[1, 2], [3, 4]],
                [
                    {mixed: "array"},
                    "with",
                    42,
                    true,
                    null
                ]
            ],
            mixed_structure: {
                level1: [
                    {
                        level2: {
                            level3: [1, 2, 3]
                        }
                    }
                ]
            }
        }
        "#;

        let start = std::time::Instant::now();
        let result = parse(input);
        let duration = start.elapsed();

        assert!(result.is_ok(), "Mixed data types should parse successfully");
        assert!(
            duration.as_millis() < 100,
            "Parsing should be fast for moderate complexity"
        );

        let parsed = result.unwrap();

        // Verify structure
        if let Value::Array(strings) = &parsed["strings"] {
            assert_eq!(strings.len(), 3);
        }
        if let Value::Array(numbers) = &parsed["numbers"] {
            assert_eq!(numbers.len(), 9);
        }
        if let Value::Array(objects) = &parsed["objects"] {
            assert_eq!(objects.len(), 3);
            assert_eq!(objects[1]["nested"]["deep"], b(true));
        }
    }
}
