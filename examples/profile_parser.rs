// this_file: examples/profile_parser.rs

use vexy_json::parse;

fn main() {
    // Heavy workload for profiling
    let complex_json = generate_complex_json();
    let forgiving_json = generate_forgiving_json();
    let string_heavy_json = generate_string_heavy_json();

    println!("Starting profiling workload...");

    // Parse complex JSON many times
    for _ in 0..1000 {
        let _ = parse(&complex_json);
    }

    // Parse forgiving features many times
    for _ in 0..2000 {
        let _ = parse(&forgiving_json);
    }

    // Parse string-heavy JSON many times
    for _ in 0..500 {
        let _ = parse(&string_heavy_json);
    }

    // Many small objects to stress allocation
    for i in 0..5000 {
        let small_obj = format!(r#"{{"id": {}, "name": "item{}", "active": true}}"#, i, i);
        let _ = parse(&small_obj);
    }

    println!("Profiling workload complete!");
}

fn generate_complex_json() -> String {
    let mut json = String::from("{");

    for i in 0..30 {
        json.push_str(&format!(
            r#""section{}": {{
                "id": {},
                "data": [
                    {{"key": "value{}", "number": {}}},
                    {{"key": "value{}", "number": {}}},
                    {{"key": "value{}", "number": {}}}
                ],
                "meta": {{
                    "created": "2023-01-{:02}",
                    "tags": ["tag1", "tag2", "tag3"],
                    "settings": {{
                        "enabled": {},
                        "threshold": {}
                    }}
                }}
            }}"#,
            i,
            i,
            i,
            i * 2,
            i,
            i * 3,
            i,
            i * 4,
            (i % 28) + 1,
            i % 2 == 0,
            i * 10
        ));

        if i < 29 {
            json.push(',');
        }
    }

    json.push('}');
    json
}

fn generate_forgiving_json() -> &'static str {
    r#"{
        // Configuration file with comments
        app_name: 'MyApp',
        version: "1.2.3",
        debug: true,
        
        /* Multi-line comment
           describing complex config */
        server: {
            host: 'localhost',
            port: 8080,
            endpoints: [
                '/api/v1',
                '/api/v2',
                '/health', // trailing comma
            ],
        },
        
        database: {
            type: 'postgres',
            url: "postgres://user:pass@localhost/db",
            pool_size: 10,
            timeout: 30000, // milliseconds
        },
        
        features: [
            'feature1',
            'feature2',
            'feature3',
        ], // trailing comma
    }"#
}

fn generate_string_heavy_json() -> String {
    let mut json = String::from("[");

    for i in 0..50 {
        json.push_str(&format!(
            r#"{{
                "title": "This is a long title for item number {} with lots of text",
                "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Item {}",
                "content": "Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. Content for item {}",
                "metadata": {{
                    "author": "Author Name {}",
                    "category": "Category {}",
                    "tags": ["tag{}", "tag{}", "tag{}"],
                    "url": "https://example.com/item/{}"
                }}
            }}"#,
            i, i, i, i, i % 10, i, (i + 1) % 10, (i + 2) % 10, i
        ));

        if i < 49 {
            json.push(',');
        }
    }

    json.push(']');
    json
}
