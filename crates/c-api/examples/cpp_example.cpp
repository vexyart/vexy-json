/**
 * @file cpp_example.cpp
 * @brief Example usage of the vexy_json C++ header-only wrapper
 */

#include <iostream>
#include <string>
#include "../include/vexy_json.hpp"

int main() {
    // Example 1: Basic parsing with default options
    try {
        std::string json = vexy_json::parse(R"({"name": "John", "age": 30})");
        std::cout << "Example 1 - Basic parsing:\n" << json << "\n\n";
    } catch (const vexy_json::ParseError& e) {
        std::cerr << "Parse error: " << e.what() << "\n";
    }
    
    // Example 2: Parsing forgiving JSON
    try {
        auto forgiving_json = R"({
            // This is a comment
            unquoted: true,
            'single': 'quotes',
            trailing: "comma",
        })";
        
        auto options = vexy_json::ParserOptions()
            .allowComments()
            .allowUnquotedKeys()
            .allowSingleQuotes()
            .allowTrailingCommas();
            
        std::string result = vexy_json::parse(forgiving_json, options);
        std::cout << "Example 2 - Forgiving JSON parsing:\n" << result << "\n\n";
    } catch (const vexy_json::ParseError& e) {
        std::cerr << "Parse error: " << e.what() << "\n";
    }
    
    // Example 3: Using a parser instance for multiple parses
    try {
        auto options = vexy_json::ParserOptions()
            .allowComments()
            .allowTrailingCommas()
            .enableRepair();
            
        vexy_json::Parser parser(options);
        
        std::vector<std::string> inputs = {
            R"({"valid": true})",
            R"({broken: true,})",
            R"({/* comment */ "key": "value"})"
        };
        
        std::cout << "Example 3 - Multiple parses with parser instance:\n";
        for (const auto& input : inputs) {
            try {
                std::string result = parser.parseToString(input);
                std::cout << "Input:  " << input << "\n";
                std::cout << "Output: " << result << "\n\n";
            } catch (const vexy_json::ParseError& e) {
                std::cout << "Failed to parse: " << e.what() << "\n\n";
            }
        }
    } catch (const std::exception& e) {
        std::cerr << "Error: " << e.what() << "\n";
    }
    
    // Example 4: Detailed parsing with repair information
    try {
        auto broken_json = R"({
            "name": "Alice"
            "age": 25,
            "city": 
        })";
        
        auto options = vexy_json::ParserOptions()
            .enableRepair()
            .reportRepairs();
            
        auto result = vexy_json::parseDetailed(broken_json, options);
        
        std::cout << "Example 4 - Detailed parsing with repairs:\n";
        std::cout << "Output: " << result.json() << "\n";
        
        if (!result.repairs().empty()) {
            std::cout << "Repairs made:\n";
            for (const auto& repair : result.repairs()) {
                std::cout << "  - " << repair.type << " at position " 
                          << repair.position << ": " << repair.description << "\n";
            }
        }
        std::cout << "\n";
    } catch (const vexy_json::ParseError& e) {
        std::cerr << "Parse error: " << e.what() << "\n";
    }
    
    // Example 5: Error handling
    try {
        std::cout << "Example 5 - Error handling:\n";
        
        auto invalid_json = R"({"unclosed": )";
        
        // This will throw
        auto options = vexy_json::ParserOptions(); // Repair disabled by default
        std::string result = vexy_json::parse(invalid_json, options);
        
    } catch (const vexy_json::ParseError& e) {
        std::cout << "Caught expected error: " << e.what() << "\n\n";
    }
    
    // Example 6: Version information
    std::cout << "Example 6 - Version information:\n";
    std::cout << "vexy_json version: " << vexy_json::version() << "\n";
    
    return 0;
}