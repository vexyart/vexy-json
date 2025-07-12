// this_file: docs/assets/js/examples.js

/**
 * Comprehensive examples system for the vexy_json web tool
 * Showcases all forgiving JSON parsing features
 */

export const EXAMPLES = {
  basic: {
    title: 'Basic JSON',
    category: 'Standard',
    description: 'Standard JSON parsing according to RFC 8259',
    content: `{
  "name": "vexy_json",
  "version": "%%VEXY_JSON_VERSION%%",
  "description": "A forgiving JSON parser for Rust",
  "features": ["forgiving", "fast", "rust"],
  "config": {
    "debug": false,
    "optimize": true
  }
}`,
    options: {
      allow_comments: false,
      allow_trailing_commas: false,
      allow_unquoted_keys: false,
      allow_single_quotes: false,
      implicit_top_level: false,
      newline_as_comma: false
    }
  },

  comments: {
    title: 'JSON with Comments',
    category: 'Comments',
    description: 'Single-line, multi-line, and hash-style comments',
    content: `{
  // Application configuration
  "name": "my-app",           // Application name
  "version": "%%VEXY_JSON_VERSION%%",         
  
  /* 
   * Server configuration block
   * Supports multi-line comments
   */
  "server": {
    "port": 8080,             // HTTP port
    "host": "localhost",      /* Bind address */
    "ssl": false,             # Enable HTTPS
    "workers": 4              # Worker processes
  },
  
  // Feature flags
  "features": {
    "auth": true,             // Enable authentication
    "cache": false,           /* Disable caching */
    "debug": true             # Debug mode
  }
}`,
    options: {
      allow_comments: true,
      allow_trailing_commas: false,
      allow_unquoted_keys: false,
      allow_single_quotes: false,
      implicit_top_level: false,
      newline_as_comma: false
    }
  },

  unquoted: {
    title: 'Unquoted Keys',
    category: 'Keys',
    description: 'Object keys without quotes, more readable configuration',
    content: `{
  name: "John Doe",
  age: 30,
  email: "john@example.com",
  address: {
    street: "123 Main St",
    city: "Anytown",
    zipCode: "12345"
  },
  preferences: {
    theme: 'dark',            // Single quotes also work
    language: "en-US",
    notifications: true
  }
}`,
    options: {
      allow_comments: true,
      allow_trailing_commas: false,
      allow_unquoted_keys: true,
      allow_single_quotes: true,
      implicit_top_level: false,
      newline_as_comma: false
    }
  },

  trailing: {
    title: 'Trailing Commas',
    category: 'Commas',
    description: 'Allow trailing commas in objects and arrays',
    content: `{
  "languages": [
    "Rust",
    "JavaScript",
    "Python",
    "Go",           // Trailing comma in array
  ],
  "frameworks": {
    "web": "React",
    "backend": "Actix",
    "database": "PostgreSQL",     // Trailing comma in object
  },
  "tools": [
    "VSCode",
    "Git",
    "Docker",       // Another trailing comma
  ],             // Even nested trailing commas work
}`,
    options: {
      allow_comments: true,
      allow_trailing_commas: true,
      allow_unquoted_keys: false,
      allow_single_quotes: false,
      implicit_top_level: false,
      newline_as_comma: false
    }
  },

  implicit: {
    title: 'Implicit Top-Level',
    category: 'Structure',
    description: 'Implicit objects and arrays without wrapping braces',
    content: `// Implicit object - no surrounding braces needed
name: "Configuration File",
version: "%%VEXY_JSON_VERSION%%",
debug: true,
database: {
  host: "localhost",
  port: 5432,
  name: "myapp"
},
features: [
  "auth",
  "caching", 
  "logging"
]`,
    options: {
      allow_comments: true,
      allow_trailing_commas: true,
      allow_unquoted_keys: true,
      allow_single_quotes: true,
      implicit_top_level: true,
      newline_as_comma: false
    }
  },

  newlines: {
    title: 'Newline as Comma',
    category: 'Commas',
    description: 'Use newlines as comma separators for cleaner syntax',
    content: `{
  // Array with newline separators
  "fruits": [
    "apple"
    "banana"      // No comma needed
    "orange"
    "grape"
  ]
  
  // Object with newline separators  
  "person": {
    "name": "Alice"
    "age": 25          // No comma needed
    "city": "Portland"
    "active": true
  }
  
  "numbers": [1, 2, 3]     // Traditional commas still work
}`,
    options: {
      allow_comments: true,
      allow_trailing_commas: true,
      allow_unquoted_keys: false,
      allow_single_quotes: false,
      implicit_top_level: false,
      newline_as_comma: true
    }
  },

  advanced: {
    title: 'All Features Combined',
    category: 'Advanced',
    description: 'Showcase of all vexy_json forgiving features together',
    content: `// Advanced configuration example
// Shows all vexy_json features working together

name: "Advanced Demo"           // Unquoted key
version: '%%VEXY_JSON_VERSION%%'               // Single quotes
debug: true                    // Newline as comma

# Server configuration
server: {
  host: 'localhost'            // Single quotes
  port: 8080                  // Newline separator
  ssl: false                  // Newline separator  
  workers: 4,                 // Traditional comma also works
}                             // Trailing comma

/* Features array with mixed syntax */
features: [
  "authentication"            // Double quotes
  'authorization'             // Single quotes  
  logging                     // Unquoted string
  caching,                    // Traditional comma
]                             // Trailing comma

// Database connections
databases: {
  primary: {
    type: postgresql          // Unquoted value
    host: "db1.example.com"  
    port: 5432               // Newline separator
    ssl: true                
  }                          // Trailing comma
  
  # Cache database
  cache: {
    type: 'redis'            // Single quotes
    host: localhost          // Unquoted value
    port: 6379
  }
}`,
    options: {
      allow_comments: true,
      allow_trailing_commas: true,
      allow_unquoted_keys: true,
      allow_single_quotes: true,
      implicit_top_level: true,
      newline_as_comma: true
    }
  }
};

/**
 * Get examples organized by category
 */
export function getExamplesByCategory() {
  const categories = {};

  Object.entries(EXAMPLES).forEach(([key, example]) => {
    const category = example.category;
    if (!categories[category]) {
      categories[category] = [];
    }
    categories[category].push({ key, ...example });
  });

  return categories;
}

/**
 * Get a specific example by key
 */
export function getExample(key) {
  return EXAMPLES[key] || null;
}

/**
 * Get all example keys
 */
export function getExampleKeys() {
  return Object.keys(EXAMPLES);
}

/**
 * Search examples by content or description
 */
export function searchExamples(query) {
  const lowercaseQuery = query.toLowerCase();

  return Object.entries(EXAMPLES)
    .filter(([key, example]) => {
      return (
        example.title.toLowerCase().includes(lowercaseQuery) ||
        example.description.toLowerCase().includes(lowercaseQuery) ||
        example.content.toLowerCase().includes(lowercaseQuery) ||
        example.category.toLowerCase().includes(lowercaseQuery)
      );
    })
    .map(([key, example]) => ({ key, ...example }));
}