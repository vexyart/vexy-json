/**
 * @file vexy_json.h
 * @brief C API for the vexy_json JSON parser
 *
 * This header provides a C-compatible API for the vexy_json JSON parser,
 * allowing integration with C/C++ applications and other language bindings.
 */

#ifndef VEXY_JSON_H
#define VEXY_JSON_H

#include <stdbool.h>
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Parser options for configuring vexy_json behavior
 */
typedef struct VexyJsonParserOptions {
    bool allow_comments;
    bool allow_trailing_commas;
    bool allow_unquoted_keys;
    bool allow_single_quotes;
    bool implicit_top_level;
    bool newline_as_comma;
    uint32_t max_depth;
    bool enable_repair;
    uint32_t max_repairs;
    bool fast_repair;
    bool report_repairs;
} VexyJsonParserOptions;

/**
 * @brief Result of parsing JSON
 */
typedef struct VexyJsonParseResult {
    char* json;     // The parsed JSON as a string (null on error)
    char* error;    // Error message (null on success)
} VexyJsonParseResult;

/**
 * @brief A single repair action
 */
typedef struct VexyJsonRepair {
    char* repair_type;
    size_t position;
    char* description;
} VexyJsonRepair;

/**
 * @brief Detailed result including repairs
 */
typedef struct VexyJsonDetailedResult {
    char* json;              // The parsed JSON as a string (null on error)
    char* error;             // Error message (null on success)
    VexyJsonRepair* repairs;   // Array of repairs made
    size_t repair_count;     // Number of repairs
} VexyJsonDetailedResult;

/**
 * @brief Opaque parser handle
 */
typedef void* VexyJsonParser;

/**
 * @brief Get the version of the vexy_json library
 * @return Version string (do not free)
 */
const char* vexy_json_version(void);

/**
 * @brief Parse JSON with default options
 * @param input The JSON string to parse (null-terminated, UTF-8)
 * @return Parse result (must be freed with vexy_json_free_result)
 * 
 * @warning This function requires careful memory management:
 * - input must be null or point to a valid null-terminated UTF-8 string
 * - The returned result must be freed using vexy_json_free_result()
 * - Do not use returned pointers after freeing the result
 */
VexyJsonParseResult vexy_json_parse(const char* input);

/**
 * @brief Parse JSON with custom options
 * @param input The JSON string to parse (null-terminated, UTF-8)
 * @param options Parser options (can be null for defaults)
 * @return Parse result (must be freed with vexy_json_free_result)
 * 
 * @warning This function requires careful memory management:
 * - input must be null or point to a valid null-terminated UTF-8 string
 * - options must be null or point to a valid VexyJsonParserOptions struct
 * - The returned result must be freed using vexy_json_free_result()
 * - Do not use returned pointers after freeing the result
 */
VexyJsonParseResult vexy_json_parse_with_options(const char* input, const VexyJsonParserOptions* options);

/**
 * @brief Parse JSON and get detailed information including repairs
 * @param input The JSON string to parse (null-terminated, UTF-8)
 * @param options Parser options (can be null for defaults)
 * @return Detailed result (must be freed with vexy_json_free_detailed_result)
 * 
 * @warning This function requires careful memory management:
 * - input must be null or point to a valid null-terminated UTF-8 string
 * - options must be null or point to a valid VexyJsonParserOptions struct
 * - The returned result must be freed using vexy_json_free_detailed_result()
 * - Do not use returned pointers after freeing the result
 */
VexyJsonDetailedResult vexy_json_parse_detailed(const char* input, const VexyJsonParserOptions* options);

/**
 * @brief Create a new parser instance
 * @param options Parser options (can be null for defaults)
 * @return Parser handle (must be freed with vexy_json_parser_free)
 * 
 * @warning This function requires careful memory management:
 * - options must be null or point to a valid VexyJsonParserOptions struct
 * - The returned parser must be freed using vexy_json_parser_free()
 * - Do not use the parser after freeing it
 * - The parser is not thread-safe; use separate instances for concurrent access
 */
VexyJsonParser vexy_json_parser_new(const VexyJsonParserOptions* options);

/**
 * @brief Parse JSON using a parser instance
 * @param parser Parser handle (created by vexy_json_parser_new)
 * @param input The JSON string to parse (null-terminated, UTF-8)
 * @return Parse result (must be freed with vexy_json_free_result)
 * 
 * @warning This function requires careful memory management:
 * - parser must be null or point to a valid parser created by vexy_json_parser_new()
 * - input must be null or point to a valid null-terminated UTF-8 string
 * - The parser must not have been freed or moved
 * - The returned result must be freed using vexy_json_free_result()
 * - Do not use returned pointers after freeing the result
 */
VexyJsonParseResult vexy_json_parser_parse(VexyJsonParser parser, const char* input);

/**
 * @brief Free a parser instance
 * @param parser Parser handle (created by vexy_json_parser_new)
 * 
 * @warning This function requires careful memory management:
 * - parser must be null or point to a valid parser created by vexy_json_parser_new()
 * - parser must not have already been freed
 * - Do not use parser after calling this function
 * - Ensure no other references to the parser exist
 */
void vexy_json_parser_free(VexyJsonParser parser);

/**
 * @brief Free a parse result
 * @param result Parse result to free
 * 
 * @warning This function requires careful memory management:
 * - result must have been returned by a parse function
 * - Do not call this function more than once for the same result
 * - Do not use result pointers after calling this function
 */
void vexy_json_free_result(VexyJsonParseResult result);

/**
 * @brief Free a detailed result
 * @param result Detailed result to free
 * 
 * @warning This function requires careful memory management:
 * - result must have been returned by vexy_json_parse_detailed()
 * - Do not call this function more than once for the same result
 * - Do not use result pointers after calling this function
 * - If repairs array is non-null, it will be properly freed
 */
void vexy_json_free_detailed_result(VexyJsonDetailedResult result);

/**
 * @brief Get default parser options
 * @return Default options
 */
VexyJsonParserOptions vexy_json_default_options(void);

#ifdef __cplusplus
}
#endif

#endif // VEXY_JSON_H