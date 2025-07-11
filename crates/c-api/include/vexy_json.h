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
 * @param input The JSON string to parse
 * @return Parse result (must be freed with vexy_json_free_result)
 */
VexyJsonParseResult vexy_json_parse(const char* input);

/**
 * @brief Parse JSON with custom options
 * @param input The JSON string to parse
 * @param options Parser options
 * @return Parse result (must be freed with vexy_json_free_result)
 */
VexyJsonParseResult vexy_json_parse_with_options(const char* input, const VexyJsonParserOptions* options);

/**
 * @brief Parse JSON and get detailed information including repairs
 * @param input The JSON string to parse
 * @param options Parser options
 * @return Detailed result (must be freed with vexy_json_free_detailed_result)
 */
VexyJsonDetailedResult vexy_json_parse_detailed(const char* input, const VexyJsonParserOptions* options);

/**
 * @brief Create a new parser instance
 * @param options Parser options
 * @return Parser handle (must be freed with vexy_json_parser_free)
 */
VexyJsonParser vexy_json_parser_new(const VexyJsonParserOptions* options);

/**
 * @brief Parse JSON using a parser instance
 * @param parser Parser handle
 * @param input The JSON string to parse
 * @return Parse result (must be freed with vexy_json_free_result)
 */
VexyJsonParseResult vexy_json_parser_parse(VexyJsonParser parser, const char* input);

/**
 * @brief Free a parser instance
 * @param parser Parser handle
 */
void vexy_json_parser_free(VexyJsonParser parser);

/**
 * @brief Free a parse result
 * @param result Parse result to free
 */
void vexy_json_free_result(VexyJsonParseResult result);

/**
 * @brief Free a detailed result
 * @param result Detailed result to free
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