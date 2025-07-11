/**
 * @file vexy_json.hpp
 * @brief C++ header-only wrapper for the vexy_json JSON parser
 *
 * This header provides a modern C++ interface for the vexy_json JSON parser,
 * with RAII, exceptions, and STL container support.
 */

#ifndef VEXY_JSON_HPP
#define VEXY_JSON_HPP

#include <string>
#include <vector>
#include <memory>
#include <stdexcept>
#include <optional>
#include <string_view>
#include <utility>

#include "vexy_json.h"

namespace vexy_json {

/**
 * @brief Exception thrown by vexy_json operations
 */
class ParseError : public std::runtime_error {
public:
    explicit ParseError(const std::string& message) 
        : std::runtime_error("vexy_json parse error: " + message) {}
};

/**
 * @brief Repair information
 */
struct Repair {
    std::string type;
    size_t position;
    std::string description;
    
    Repair(const VexyJsonRepair& r) 
        : type(r.repair_type ? r.repair_type : ""),
          position(r.position),
          description(r.description ? r.description : "") {}
};

/**
 * @brief Parser options wrapper
 */
class ParserOptions {
public:
    ParserOptions() : options_(vexy_json_default_options()) {}
    
    ParserOptions& allowComments(bool value = true) {
        options_.allow_comments = value;
        return *this;
    }
    
    ParserOptions& allowTrailingCommas(bool value = true) {
        options_.allow_trailing_commas = value;
        return *this;
    }
    
    ParserOptions& allowUnquotedKeys(bool value = true) {
        options_.allow_unquoted_keys = value;
        return *this;
    }
    
    ParserOptions& allowSingleQuotes(bool value = true) {
        options_.allow_single_quotes = value;
        return *this;
    }
    
    ParserOptions& implicitTopLevel(bool value = true) {
        options_.implicit_top_level = value;
        return *this;
    }
    
    ParserOptions& newlineAsComma(bool value = true) {
        options_.newline_as_comma = value;
        return *this;
    }
    
    ParserOptions& maxDepth(uint32_t depth) {
        options_.max_depth = depth;
        return *this;
    }
    
    ParserOptions& enableRepair(bool value = true) {
        options_.enable_repair = value;
        return *this;
    }
    
    ParserOptions& maxRepairs(uint32_t count) {
        options_.max_repairs = count;
        return *this;
    }
    
    ParserOptions& fastRepair(bool value = true) {
        options_.fast_repair = value;
        return *this;
    }
    
    ParserOptions& reportRepairs(bool value = true) {
        options_.report_repairs = value;
        return *this;
    }
    
    const vexy_json_parser_options* get() const { return &options_; }
    
private:
    vexy_json_parser_options options_;
};

/**
 * @brief Parse result wrapper
 */
class ParseResult {
public:
    ParseResult() = default;
    
    explicit ParseResult(vexy_json_parse_result result) 
        : result_(std::make_unique<vexy_json_parse_result>(result)) {
        if (result.error) {
            error_ = result.error;
        }
        if (result.json) {
            json_ = result.json;
        }
    }
    
    ParseResult(ParseResult&& other) noexcept = default;
    ParseResult& operator=(ParseResult&& other) noexcept = default;
    
    ParseResult(const ParseResult&) = delete;
    ParseResult& operator=(const ParseResult&) = delete;
    
    ~ParseResult() {
        if (result_) {
            vexy_json_free_result(*result_);
        }
    }
    
    bool hasError() const { return error_.has_value(); }
    
    const std::string& error() const {
        if (!error_) {
            throw std::logic_error("No error present");
        }
        return *error_;
    }
    
    const std::string& json() const {
        if (!json_) {
            throw ParseError(error_.value_or("Unknown error"));
        }
        return *json_;
    }
    
    std::string json() {
        if (!json_) {
            throw ParseError(error_.value_or("Unknown error"));
        }
        return std::move(*json_);
    }
    
private:
    std::unique_ptr<vexy_json_parse_result> result_;
    std::optional<std::string> json_;
    std::optional<std::string> error_;
};

/**
 * @brief Detailed parse result with repair information
 */
class DetailedParseResult {
public:
    DetailedParseResult() = default;
    
    explicit DetailedParseResult(vexy_json_detailed_result result) 
        : result_(std::make_unique<vexy_json_detailed_result>(result)) {
        if (result.error) {
            error_ = result.error;
        }
        if (result.json) {
            json_ = result.json;
        }
        if (result.repairs && result.repair_count > 0) {
            repairs_.reserve(result.repair_count);
            for (size_t i = 0; i < result.repair_count; ++i) {
                repairs_.emplace_back(result.repairs[i]);
            }
        }
    }
    
    DetailedParseResult(DetailedParseResult&& other) noexcept = default;
    DetailedParseResult& operator=(DetailedParseResult&& other) noexcept = default;
    
    DetailedParseResult(const DetailedParseResult&) = delete;
    DetailedParseResult& operator=(const DetailedParseResult&) = delete;
    
    ~DetailedParseResult() {
        if (result_) {
            vexy_json_free_detailed_result(*result_);
        }
    }
    
    bool hasError() const { return error_.has_value(); }
    
    const std::string& error() const {
        if (!error_) {
            throw std::logic_error("No error present");
        }
        return *error_;
    }
    
    const std::string& json() const {
        if (!json_) {
            throw ParseError(error_.value_or("Unknown error"));
        }
        return *json_;
    }
    
    const std::vector<Repair>& repairs() const { return repairs_; }
    
private:
    std::unique_ptr<vexy_json_detailed_result> result_;
    std::optional<std::string> json_;
    std::optional<std::string> error_;
    std::vector<Repair> repairs_;
};

/**
 * @brief Main parser class
 */
class Parser {
public:
    Parser() : Parser(ParserOptions{}) {}
    
    explicit Parser(const ParserOptions& options) 
        : parser_(vexy_json_parser_new(options.get())) {
        if (!parser_) {
            throw std::runtime_error("Failed to create vexy_json parser");
        }
    }
    
    Parser(Parser&& other) noexcept : parser_(other.parser_) {
        other.parser_ = nullptr;
    }
    
    Parser& operator=(Parser&& other) noexcept {
        if (this != &other) {
            if (parser_) {
                vexy_json_parser_free(parser_);
            }
            parser_ = other.parser_;
            other.parser_ = nullptr;
        }
        return *this;
    }
    
    Parser(const Parser&) = delete;
    Parser& operator=(const Parser&) = delete;
    
    ~Parser() {
        if (parser_) {
            vexy_json_parser_free(parser_);
        }
    }
    
    ParseResult parse(std::string_view input) const {
        std::string input_str(input);
        return ParseResult(vexy_json_parser_parse(parser_, input_str.c_str()));
    }
    
    std::string parseToString(std::string_view input) const {
        auto result = parse(input);
        return result.json();
    }
    
private:
    vexy_json_parser parser_;
};

/**
 * @brief Convenience functions for quick parsing
 */
inline std::string parse(std::string_view input) {
    std::string input_str(input);
    auto result = ParseResult(vexy_json_parse(input_str.c_str()));
    return result.json();
}

inline std::string parse(std::string_view input, const ParserOptions& options) {
    std::string input_str(input);
    auto result = ParseResult(vexy_json_parse_with_options(input_str.c_str(), options.get()));
    return result.json();
}

inline DetailedParseResult parseDetailed(std::string_view input, const ParserOptions& options) {
    std::string input_str(input);
    return DetailedParseResult(vexy_json_parse_detailed(input_str.c_str(), options.get()));
}

/**
 * @brief Get the version of the vexy_json library
 */
inline std::string version() {
    return vexy_json_version();
}

} // namespace vexy_json

#endif // VEXY_JSON_HPP