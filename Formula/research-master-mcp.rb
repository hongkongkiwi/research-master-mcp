class ResearchMasterMcp < Formula
  desc "MCP server for searching and downloading academic papers from multiple research sources"
  homepage "https://github.com/hongkongkiwi/research-master-mcp"
  license "MIT"
  version "0.1.0"

  on_macos do
    on_arm do
      url "https://github.com/hongkongkiwi/research-master-mcp/releases/download/v#{version}/research-master-mcp-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "TODO: Update with actual sha256 hash"
    end
    on_intel do
      url "https://github.com/hongkongkiwi/research-master-mcp/releases/download/v#{version}/research-master-mcp-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "TODO: Update with actual sha256 hash"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/hongkongkiwi/research-master-mcp/releases/download/v#{version}/research-master-mcp-v#{version}-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "TODO: Update with actual sha256 hash"
    end
    on_x86_64 do
      url "https://github.com/hongkongkiwi/research-master-mcp/releases/download/v#{version}/research-master-mcp-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "TODO: Update with actual sha256 hash"
    end
  end

  depends_on "poppler" => :optional

  def install
    bin.install "research-master-mcp"

    # Install example configuration
    (etc/"research-master").install "research-master.example.toml"
  end

  def caveats
    <<~EOS
      Configuration file is located at:
        #{etc}/research-master/research-master.toml

      To use with Claude Desktop, add to your MCP config:
        {
          "mcpServers": {
            "research-master": {
              "command": "#{bin}/research-master-mcp",
              "args": ["serve"]
            }
          }
        }
    EOS
  end

  test do
    # Basic version check
    assert_match version.to_s, shell_output("#{bin}/research-master-mcp --version").strip
  end
end
