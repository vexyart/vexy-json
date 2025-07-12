class VexyJson < Formula
  desc "Forgiving JSON parser for Rust - a port of the JavaScript library jsonic"
  homepage "https://github.com/vexyart/vexy-json"
  url "https://github.com/vexyart/vexy-json/archive/refs/tags/v2.0.0.tar.gz"
  sha256 "ce66e4af1e0aeb4f35456eb44aa82d5052e1a26c33adbaa1969284a5aa8c24ab"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/vexyart/vexy-json.git", branch: "main"

  depends_on "rust" => :build

  def install
    cd "crates/cli" do
      system "cargo", "install", *std_cargo_args
    end
  end

  test do
    # Test basic JSON parsing
    assert_equal '{"key":"value"}', pipe_output("#{bin}/vexy-json", '{"key": "value"}').chomp

    # Test forgiving JSON features
    forgiving_json = '{ unquoted: true, trailing: "comma", }'
    output = pipe_output("#{bin}/vexy-json", forgiving_json)
    assert_match /"unquoted":true/, output
    assert_match /"trailing":"comma"/, output

    # Test error repair
    broken_json = '{ "broken": '
    output = pipe_output("#{bin}/vexy-json --repair", broken_json)
    assert_match /"broken":null/, output

    # Test version
    assert_match version.to_s, shell_output("#{bin}/vexy-json --version")
  end
end